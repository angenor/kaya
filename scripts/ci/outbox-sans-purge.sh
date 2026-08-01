#!/usr/bin/env bash
#
# **Porte P-05b** — aucun chemin de code ne supprime ni ne purge un **registre immuable**.
#
# # Une catégorie, plus une liste de tables (constitution 1.6.0)
#
# La porte portait sur `evenement_outbox` seule. Le cycle 003 lui donne un second registre —
# `comptes.journal_audit` — et le choix a été fait de reformuler la porte plutôt que d'allonger
# une liste : elle porte désormais sur la **catégorie**, de sorte que le prochain registre soit
# couvert sans nouvel amendement. Un troisième s'ajoute au tableau `REGISTRES` ci-dessous ; rien
# d'autre ne bouge.
#
# # La troisième couche de l'immuabilité (R-05)
#
# Les deux autres vivent dans la base : le `REVOKE` — ou, pour le journal d'audit, l'absence pure
# et simple du privilège — arrête le bug applicatif, et le déclencheur arrête la maintenance
# lancée sous le propriétaire. Celle-ci arrête **le code qui aurait été écrit pour purger**, avant
# qu'il n'atteigne une base.
#
# Elle compte parce que la pression est réelle et viendra : les deux tables croissent sans fin,
# quelqu'un proposera une rétention, et l'argument sera « on garde deux ans, ça suffit ».
#
#   * le **grand livre** a une rétention illimitée (TRX-02) : la génération SYSCOHADA rétroactive
#     de la phase 2 relira des événements de 2026, et les documents fiscaux se conservent dix ans ;
#   * le **registre des actions** est ce que le propriétaire achète (CPT-04). Un registre dont on
#     peut effacer les six derniers mois ne prouve rien — c'est-à-dire ne sert à rien.
#
# # Le versant que ce script ne porte PAS, et qui compte autant
#
# Ce contrôle est **négatif** : il constate l'absence de chemins de purge. À lui seul, il passerait
# au vert si les tables disparaissaient — une porte qui ne trouve jamais rien est indistinguable
# d'une porte qui n'a rien à trouver.
#
# Son versant **positif** vit dans les tests : `backend/tests/outbox_immuabilite.rs` et
# `backend/tests/audit_immuabilite.rs` écrivent réellement une ligne, la relisent, et constatent
# que ni `UPDATE` ni `DELETE` n'aboutissent. Les deux moitiés sont exigées par le § « Couverture
# des portes ».

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

echo "── P-05b — aucun chemin de purge d'un registre immuable ──────────────────────"

echec=0

# Les registres immuables du produit, sous la forme « schéma.table|fichier de test exclu ».
#
# Le test d'immuabilité de chaque registre **tente** un `DELETE` — c'est sa raison d'être :
# constater le refus. Il est donc exclu, nommément, et lui seul. Chaque registre porte son
# exclusion à côté de son nom pour qu'on ne puisse pas ajouter l'une sans l'autre.
REGISTRES=(
    "synchronisation.evenement_outbox|backend/tests/outbox_immuabilite.rs"
    "comptes.journal_audit|backend/tests/audit_immuabilite.rs"
)

echo "  ${#REGISTRES[@]} registre(s) inspecté(s) :"
for entree in "${REGISTRES[@]}"; do
    echo "      · ${entree%%|*}"
done

for entree in "${REGISTRES[@]}"; do
    qualifie="${entree%%|*}"          # synchronisation.evenement_outbox
    test_exclu="${entree##*|}"        # backend/tests/outbox_immuabilite.rs
    nu="${qualifie##*.}"              # evenement_outbox

    MOTIFS=(
        "DELETE[[:space:]]+FROM[[:space:]]+${qualifie//./\\.}"
        "DELETE[[:space:]]+FROM[[:space:]]+${nu}"
        "TRUNCATE[[:space:]]+.*${nu}"
        "DROP[[:space:]]+TABLE[[:space:]]+.*${nu}"
    )

    for motif in "${MOTIFS[@]}"; do
        # `REVOKE ... DELETE, TRUNCATE` et le déclencheur qui refuse la suppression sont ce qui
        # PROTÈGE la table : les compter comme des chemins de purge ferait échouer la porte sur sa
        # propre défense — et la première réaction serait de la désactiver.
        resultats="$(grep -rniE "$motif" \
            --include='*.rs' --include='*.sql' --include='*.sh' \
            backend scripts infra 2>/dev/null \
            | grep -v "$test_exclu" \
            | grep -v 'scripts/ci/outbox-sans-purge.sh' \
            | grep -viE '^[^:]*:[0-9]+:[[:space:]]*(REVOKE|GRANT)\b' \
            | grep -viE "TG_OP[[:space:]]*=[[:space:]]*'DELETE'" || true)"

        if [[ -n "$resultats" ]]; then
            echo "  ✗ chemin de suppression détecté sur $qualifie :" >&2
            echo "$resultats" | sed 's/^/      /' >&2
            echec=1
        fi
    done
done

# Rétention bornée : la formulation trahit l'intention avant que le code n'existe.
retention="$(grep -rniE '(retention|purge|archivage|nettoyage).{0,40}(evenement_outbox|journal_audit|grand.livre|outbox|registre des actions)' \
    --include='*.rs' --include='*.sql' --include='*.sh' --include='*.yml' \
    backend scripts infra .github 2>/dev/null \
    | grep -v 'scripts/ci/outbox-sans-purge.sh' \
    | grep -viE 'rétention illimitée|retention illimitee|jamais de suppression|interdite|aucune purge|aucun chemin' || true)"

if [[ -n "$retention" ]]; then
    echo "  ✗ mention d'une rétention ou d'une purge d'un registre immuable :" >&2
    echo "$retention" | sed 's/^/      /' >&2
    echec=1
fi

if [[ $echec -ne 0 ]]; then
    echo >&2
    echo "P-05b ÉCHOUE — un registre immuable ne se purge JAMAIS (principe II, TRX-02, CPT-04)." >&2
    echo "Une correction est une NOUVELLE ligne, jamais une suppression." >&2
    echo "  · grand livre : la génération SYSCOHADA rétroactive relira des événements de 2026," >&2
    echo "    et les documents fiscaux se conservent dix ans." >&2
    echo "  · registre des actions : c'est ce que le propriétaire achète. Un registre dont on" >&2
    echo "    peut effacer les six derniers mois ne prouve rien." >&2
    exit 1
fi

echo "P-05b ✓ — aucun chemin de suppression ni de rétention bornée sur ${#REGISTRES[@]} registre(s)."
