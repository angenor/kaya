#!/usr/bin/env bash
#
# **Porte P-19** — aucun fichier de `docs/design/html/` copié sous `app/`.
#
# # Pourquoi la copie est interdite
#
# Le HTML de maquette est une **cible, pas une source** (principe XII). Il est autonome, non
# sémantique, sans i18n, sans mode sombre câblé, sans RBAC. Le copier importe tout cela dans
# l'application, et il faudra le défaire **écran par écran** — réimplémenter depuis des valeurs
# exactes coûte moins cher que corriger une copie.
#
# On lit ses valeurs, on réimplémente.
#
# # La seule exception, et elle est vérifiée
#
# `docs/design/theme.css` est copié **tel quel** dans `app/assets/css/` : c'est lui qui porte les
# jetons dans le code. La porte l'exclut explicitement — et vérifie en plus qu'il s'agit bien d'une
# copie **identique**. Une copie divergente serait pire qu'une absence de copie : les jetons de
# l'application ne seraient plus ceux du design, sans que rien ne le signale.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

SOURCE="docs/design"
CIBLE="app"

echo "── P-19 — aucune maquette copiée sous app/ ───────────────────────────────────"

if [[ ! -d "$SOURCE/html" ]]; then
    echo "P-19 ÉCHOUE — $SOURCE/html est introuvable : la porte n'a rien à comparer." >&2
    exit 1
fi

empreinte() {
    if command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | cut -d' ' -f1
    else sha256sum "$1" | cut -d' ' -f1; fi
}

# --- Empreintes des maquettes ------------------------------------------------------------------
declare -a EMPREINTES_MAQUETTES=()
declare -a NOMS_MAQUETTES=()
while IFS= read -r fichier; do
    EMPREINTES_MAQUETTES+=("$(empreinte "$fichier")")
    NOMS_MAQUETTES+=("$fichier")
done < <(find "$SOURCE/html" "$SOURCE/fondation" "$SOURCE/proto" "$SOURCE/documents" \
    -type f \( -name '*.html' -o -name '*.css' \) 2>/dev/null | sort)

echo "  ${#EMPREINTES_MAQUETTES[@]} fichier(s) de maquette"

echec=0

while IFS= read -r fichier; do
    relatif="${fichier#./}"

    # Seule exception du principe XII.
    if [[ "$relatif" == "app/assets/css/theme.css" ]]; then
        continue
    fi

    empreinte_cible="$(empreinte "$fichier")"
    for index in "${!EMPREINTES_MAQUETTES[@]}"; do
        if [[ "$empreinte_cible" == "${EMPREINTES_MAQUETTES[$index]}" ]]; then
            echo "  ✗ $relatif est une copie de ${NOMS_MAQUETTES[$index]}" >&2
            echec=1
        fi
    done
done < <(find "$CIBLE" -type f \( -name '*.html' -o -name '*.vue' -o -name '*.css' \) \
    -not -path '*/node_modules/*' -not -path '*/.nuxt/*' -not -path '*/.output/*' 2>/dev/null | sort)

# --- L'exception doit rester une copie EXACTE ---------------------------------------------------
if [[ -f "app/assets/css/theme.css" ]]; then
    if ! cmp -s "$SOURCE/theme.css" "app/assets/css/theme.css"; then
        echo "  ✗ app/assets/css/theme.css DIVERGE de $SOURCE/theme.css" >&2
        echo "      C'est la seule copie autorisée du principe XII, et elle doit être exacte." >&2
        echo "      Une copie divergente est pire qu'une absence : les jetons de l'application" >&2
        echo "      ne sont plus ceux du design, et rien ne le signale." >&2
        echec=1
    else
        echo "  · app/assets/css/theme.css — copie exacte vérifiée (seule exception)"
    fi
else
    echo "  ✗ app/assets/css/theme.css est absent : le thème ne porte aucun jeton." >&2
    echec=1
fi

if [[ $echec -ne 0 ]]; then
    echo >&2
    echo "P-19 ÉCHOUE — le HTML de maquette est une CIBLE, pas une source (principe XII)." >&2
    echo "On lit ses valeurs, on réimplémente." >&2
    exit 1
fi

echo "P-19 ✓ — aucune maquette copiée, exception conforme."
