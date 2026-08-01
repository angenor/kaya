#!/usr/bin/env bash
#
# **Porte P-10** — aucun montant non entier ; aucune quantité non `NUMERIC`.
#
# # Deux fautes distinctes, et pourquoi elles sont irrattrapables après coup
#
# **Un montant en flottant** produit des erreurs d'arrondi qui se répliquent chez tous les clients.
# Sur des documents fiscaux, l'écart n'est pas une gêne : c'est un redressement.
#
# **Une quantité en entier** ferme la porte à tout ce qui ne se compte pas à l'unité. Un hôtel vend
# 1 bière ; une quincaillerie vendra 2,3 mètres de fer, une boulangerie achètera 47,5 kg de farine.
# Passer d'entier à décimal **après mise en production** imposerait de migrer toutes les lignes de
# vente et tous les mouvements de stock de tous les clients.
#
# # Le JSONB, où le principe V cessait de tenir (constitution 1.6.0)
#
# Un document JSON accepte `12500.5` ou `"12 500 F"` là où le principe impose un entier d'unité
# mineure — et le registre concerné, `comptes.journal_audit`, trace précisément les **écarts de
# caisse**, les **modifications de tarif** et les **remises**, c'est-à-dire les trois choses qu'un
# propriétaire consulte pour détecter une fraude. Un écart stocké en flottant, et l'audit ment sur
# le montant qu'il est censé prouver.
#
# La convention, vérifiée aux sections 4 et 5 :
#
#     { "ecart_mineur": -12500, "devise": "XOF", "motif": "…" }
#
# **Ce contrôle statique ne voit pas tout, et c'est écrit ici plutôt que supposé.** Un document
# construit dynamiquement par un service — `json!({ cle: valeur })` — lui échappe entièrement.
# C'est pourquoi `socle/comptes/src/audit/service.rs::valider_contexte` le double **à l'écriture**.
# Réciproquement, cette validation à l'exécution ne voit pas un littéral mal nommé dans du code qui
# ne s'exécute pas encore. **Les deux sont nécessaires ; aucun ne remplace l'autre.**
#
# # Portée réelle à ce cycle : presque à vide, et c'est dit
#
# Aucune valeur monétaire n'est persistée hors du jeu de cas figé du test de reconstitution
# (data-model §9). La porte est donc installée avec son **assertion de non-régression** (R-15) :
# elle échouera le jour où elle cesserait de trouver des migrations à analyser — c'est-à-dire le
# jour où elle serait devenue inopérante sans que personne ne s'en aperçoive.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

MIGRATIONS="backend/migrations"
# Le code Rust où se construisent les documents JSON d'audit et d'outbox.
SOURCES_RUST="backend/crates backend/api/src"

echo "── P-10 — montants entiers, quantités NUMERIC, JSONB compris ─────────────────"

fichiers=$(find "$MIGRATIONS" -maxdepth 1 -name '*.sql' 2>/dev/null | wc -l | tr -d ' ')

# Assertion de non-régression (R-15).
if [[ "$fichiers" -eq 0 ]]; then
    echo "P-10 ÉCHOUE — aucune migration à analyser." >&2
    echo "La porte ne vérifie plus rien. Si les migrations ont changé d'emplacement," >&2
    echo "ce script doit être mis à jour dans le MÊME changement." >&2
    exit 1
fi
sources=$(find $SOURCES_RUST -name '*.rs' 2>/dev/null | wc -l | tr -d ' ')

# Seconde assertion de non-régression : le périmètre Rust, ajouté par la constitution 1.6.0.
# Sans elle, un déplacement de `backend/crates` rendrait les sections 4 et 5 muettes, et la porte
# continuerait d'afficher son ✓ en n'inspectant plus que le SQL.
if [[ "$sources" -eq 0 ]]; then
    echo "P-10 ÉCHOUE — aucun fichier Rust à analyser sous $SOURCES_RUST." >&2
    echo "Les contrôles JSONB (sections 4 et 5) ne vérifieraient plus rien." >&2
    exit 1
fi
echo "  $fichiers migration(s) et $sources fichier(s) Rust analysé(s)"

# Corps de production de chaque fichier Rust : sans les commentaires, et **tronqué au premier
# `#[cfg(test)]`**.
#
# Les deux exclusions sont nécessaires, et aucune n'est une commodité :
#
#   * les **commentaires** décrivent souvent la forme interdite pour expliquer pourquoi elle
#     l'est — `audit/service.rs` documente `{"montant": 12500}` comme le contre-exemple ;
#   * les **modules de test** l'exercent réellement : `un_montant_decimal_est_refuse` construit
#     `{"ecart_mineur": -12500.5}` et vérifie qu'il est refusé. Sans cette troncature, la porte
#     ferait échouer le build à cause des tests qui prouvent qu'elle a raison.
#
# La troncature au premier `#[cfg(test)]` suppose la convention du dépôt : les tests unitaires
# vivent en fin de fichier, dans un unique `mod tests`. Un `#[cfg(test)]` posé en tête masquerait
# le fichier entier — d'où le décompte des lignes réellement inspectées, ci-dessous.
corps_production() {
    find $SOURCES_RUST -name '*.rs' 2>/dev/null -print0 \
        | xargs -0 awk '
            FNR == 1 { en_test = 0 }
            /^[[:space:]]*#\[cfg\(test\)\]/ { en_test = 1 }
            en_test { next }
            /^[[:space:]]*\/\// { next }
            { print FILENAME ":" FNR ":" $0 }
        '
}

lignes_production=$(corps_production | wc -l | tr -d ' ')
if [[ "$lignes_production" -eq 0 ]]; then
    echo "P-10 ÉCHOUE — aucune ligne de code de production Rust inspectée." >&2
    echo "La troncature au premier « #[cfg(test)] » a-t-elle avalé les fichiers entiers ?" >&2
    exit 1
fi
echo "  $lignes_production ligne(s) de code de production Rust inspectée(s)"

echec=0

# --- 1. Montants en type approché --------------------------------------------------------------
montants="$(grep -rniE '^[^-]*\b[a-z_]*(montant|prix|total|solde|somme|tarif|cout)[a-z_]*\b[[:space:]]+(FLOAT|REAL|DOUBLE|DECIMAL\(|MONEY)' \
    "$MIGRATIONS" 2>/dev/null || true)"
if [[ -n "$montants" ]]; then
    echo "  ✗ montant en type approché :" >&2
    echo "$montants" | sed 's/^/      /' >&2
    echec=1
fi

# --- 2. Quantités en entier --------------------------------------------------------------------
quantites="$(grep -rniE '^[^-]*\b[a-z_]*(quantite|qte)[a-z_]*\b[[:space:]]+(INT|INTEGER|BIGINT|SMALLINT)\b' \
    "$MIGRATIONS" 2>/dev/null || true)"
if [[ -n "$quantites" ]]; then
    echo "  ✗ quantité en entier — un hôtel vend 1 bière, une quincaillerie 2,3 m de fer :" >&2
    echo "$quantites" | sed 's/^/      /' >&2
    echec=1
fi

# --- 3. Taux en flottant ------------------------------------------------------------------------
# Les taux sont en millièmes entiers (180 = 18 %). Un taux flottant rouvrirait par la petite porte
# le risque d'arrondi que le principe V ferme sur les montants.
taux="$(grep -rniE '^[^-]*\b[a-z_]*(taux|tva)[a-z_]*\b[[:space:]]+(FLOAT|REAL|DOUBLE)' \
    "$MIGRATIONS" 2>/dev/null || true)"
if [[ -n "$taux" ]]; then
    echo "  ✗ taux en flottant — les taux sont en MILLIÈMES ENTIERS (180 = 18 %) :" >&2
    echo "$taux" | sed 's/^/      /' >&2
    echec=1
fi

# --- 4. Clés monétaires JSONB à valeur non entière ---------------------------------------------
#
# Cherche les littéraux `"…_mineur": <non-entier>` dans les documents JSON construits en Rust.
# Un décimal (`12500.5`) ou une chaîne (`"12 500 F"`) sont les deux formes que le JSONB accepte
# sans broncher et que le principe V interdit.
jsonb_non_entier="$(corps_production | grep -E '"[a-z_]+_mineur"[[:space:]]*:[[:space:]]*("|[-0-9]+\.)' || true)"
if [[ -n "$jsonb_non_entier" ]]; then
    echo "  ✗ clé monétaire JSONB à valeur non entière — un montant est un ENTIER d'unité mineure :" >&2
    echo "$jsonb_non_entier" | sed 's/^/      /' >&2
    echec=1
fi

# --- 5. Montants JSONB nommés nus --------------------------------------------------------------
#
# `{"montant": 12500}` : rien ne dit si 12500 est en unités ou en centimes. Le nombre de décimales
# vient de la DEVISE (principe V), et le suffixe réservé est ce qui rend l'entier interprétable six
# mois plus tard.
#
# La liste des noms suit `NOMS_MONETAIRES_NUS` de `audit/service.rs` — les deux se tiennent, et
# c'est la validation à l'écriture qui fait autorité sur les documents dynamiques.
jsonb_nu="$(corps_production | grep -E '"(montant|prix|total|somme|cout)"[[:space:]]*:' || true)"
if [[ -n "$jsonb_nu" ]]; then
    echo "  ✗ montant JSONB nommé nu — employer « <nom>_mineur » avec « devise » au même niveau :" >&2
    echo "$jsonb_nu" | sed 's/^/      /' >&2
    echec=1
fi

if [[ $echec -ne 0 ]]; then
    echo >&2
    echo "P-10 ÉCHOUE (principe V)." >&2
    echo "  Montants : ENTIERS d'unité mineure + code ISO 4217 porté par l'établissement." >&2
    echo "  Quantités : NUMERIC, JAMAIS entier." >&2
    echo "  JSONB     : clé « <nom>_mineur », valeur ENTIÈRE, clé « devise » au même niveau." >&2
    echo "Corriger maintenant coûte une migration ; après mise en production, il faut migrer" >&2
    echo "toutes les lignes de tous les clients." >&2
    exit 1
fi

echo "P-10 ✓ — aucun montant approché, aucune quantité entière, aucun taux flottant,"
echo "        aucune clé monétaire JSONB non entière ni nommée nue."
