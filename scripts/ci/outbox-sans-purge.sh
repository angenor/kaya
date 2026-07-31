#!/usr/bin/env bash
#
# **Porte P-05b** — aucun chemin de code ne supprime ni ne purge un événement du grand livre.
#
# # La troisième couche de l'immuabilité (R-05)
#
# Les deux autres vivent dans la base : le `REVOKE` arrête le bug applicatif, le déclencheur
# arrête la maintenance lancée sous le propriétaire. Celle-ci arrête **le code qui aurait été
# écrit pour purger** — avant qu'il n'atteigne une base.
#
# Elle compte parce que la pression est réelle et viendra : la table croît sans fin, quelqu'un
# proposera une rétention, et l'argument sera « on garde deux ans, ça suffit ». Le grand livre a
# une **rétention illimitée** (TRX-02) : la génération SYSCOHADA rétroactive de la phase 2 relira
# des événements de 2026, et les documents fiscaux se conservent dix ans.
#
# L'index partiel sur `publie_le IS NULL` est ce qui rend cette rétention tenable sans dégrader le
# worker : la table grandit, l'index de travail non.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

echo "── P-05b — aucun chemin de purge du grand livre ──────────────────────────────"

echec=0

# `DELETE`/`TRUNCATE` visant la table, et les mots qui trahissent une intention de rétention.
MOTIFS=(
    'DELETE[[:space:]]+FROM[[:space:]]+synchronisation\.evenement_outbox'
    'DELETE[[:space:]]+FROM[[:space:]]+evenement_outbox'
    'TRUNCATE[[:space:]]+.*evenement_outbox'
    'DROP[[:space:]]+TABLE[[:space:]]+.*evenement_outbox'
)

for motif in "${MOTIFS[@]}"; do
    # Le test d'immuabilité TENTE un DELETE — c'est sa raison d'être : constater le refus. Il est
    # donc exclu, nommément, et lui seul.
    # `REVOKE ... DELETE, TRUNCATE` et le déclencheur qui refuse la suppression sont ce qui
    # PROTÈGE la table : les compter comme des chemins de purge ferait échouer la porte sur sa
    # propre défense — et la première réaction serait de la désactiver.
    resultats="$(grep -rniE "$motif" \
        --include='*.rs' --include='*.sql' --include='*.sh' \
        backend scripts infra 2>/dev/null \
        | grep -v 'backend/tests/outbox_immuabilite.rs' \
        | grep -v 'scripts/ci/outbox-sans-purge.sh' \
        | grep -viE '^[^:]*:[0-9]+:[[:space:]]*(REVOKE|GRANT)\b' \
        | grep -viE "TG_OP[[:space:]]*=[[:space:]]*'DELETE'" || true)"

    if [[ -n "$resultats" ]]; then
        echo "  ✗ chemin de suppression détecté :" >&2
        echo "$resultats" | sed 's/^/      /' >&2
        echec=1
    fi
done

# Rétention bornée : la formulation trahit l'intention avant que le code n'existe.
retention="$(grep -rniE '(retention|purge|archivage|nettoyage).{0,40}(evenement_outbox|grand.livre|outbox)' \
    --include='*.rs' --include='*.sql' --include='*.sh' --include='*.yml' \
    backend scripts infra .github 2>/dev/null \
    | grep -v 'scripts/ci/outbox-sans-purge.sh' \
    | grep -viE 'rétention illimitée|retention illimitee|jamais de suppression|interdite|aucune purge|aucun chemin' || true)"

if [[ -n "$retention" ]]; then
    echo "  ✗ mention d'une rétention ou d'une purge du grand livre :" >&2
    echo "$retention" | sed 's/^/      /' >&2
    echec=1
fi

if [[ $echec -ne 0 ]]; then
    echo >&2
    echo "P-05b ÉCHOUE — le grand livre a une RÉTENTION ILLIMITÉE (TRX-02, principe II)." >&2
    echo "Une correction est un NOUVEL événement, jamais une suppression." >&2
    echo "La génération SYSCOHADA rétroactive relira des événements de 2026, et les documents" >&2
    echo "fiscaux se conservent dix ans." >&2
    exit 1
fi

echo "P-05b ✓ — aucun chemin de suppression ni de rétention bornée."
