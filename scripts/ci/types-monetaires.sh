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

echo "── P-10 — montants entiers, quantités NUMERIC ────────────────────────────────"

fichiers=$(find "$MIGRATIONS" -maxdepth 1 -name '*.sql' 2>/dev/null | wc -l | tr -d ' ')

# Assertion de non-régression (R-15).
if [[ "$fichiers" -eq 0 ]]; then
    echo "P-10 ÉCHOUE — aucune migration à analyser." >&2
    echo "La porte ne vérifie plus rien. Si les migrations ont changé d'emplacement," >&2
    echo "ce script doit être mis à jour dans le MÊME changement." >&2
    exit 1
fi
echo "  $fichiers migration(s) analysée(s)"

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

if [[ $echec -ne 0 ]]; then
    echo >&2
    echo "P-10 ÉCHOUE (principe V)." >&2
    echo "  Montants : ENTIERS d'unité mineure + code ISO 4217 porté par l'établissement." >&2
    echo "  Quantités : NUMERIC, JAMAIS entier." >&2
    echo "Corriger maintenant coûte une migration ; après mise en production, il faut migrer" >&2
    echo "toutes les lignes de tous les clients." >&2
    exit 1
fi

echo "P-10 ✓ — aucun montant approché, aucune quantité entière, aucun taux flottant."
