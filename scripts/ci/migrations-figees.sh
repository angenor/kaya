#!/usr/bin/env bash
#
# **Porte P-02** — aucune migration déjà appliquée n'a été modifiée.
#
# # Pourquoi c'est la porte la plus impitoyable du jeu
#
# Une migration appliquée décrit un état que des bases **portent déjà**. La modifier ne change
# rien à ces bases : elle crée une divergence entre ce que le dépôt décrit et ce que la production
# contient — divergence invisible jusqu'au jour où une base neuve est créée et se retrouve avec un
# schéma différent de toutes les autres.
#
# Le parc auto-hébergé rend cela irrattrapable : les migrations tournent au démarrage, chez des
# clients, sans que personne ne regarde. Une correction se fait donc **toujours par une migration
# nouvelle**, jamais par édition.
#
# # Le mécanisme
#
# Empreinte de chaque fichier de migration, comparée à celle de la **branche de base**. Un fichier
# existant qui diffère fait échouer la porte ; un fichier ajouté est normal.
#
# Sur une branche sans base de comparaison — premier commit d'un dépôt neuf — la porte ne peut
# rien vérifier et le dit, plutôt que de passer au vert en silence.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

MIGRATIONS="backend/migrations"
base="${GITHUB_BASE_REF:-main}"

echo "── P-02 — migrations figées ──────────────────────────────────────────────────"

if [[ ! -d "$MIGRATIONS" ]]; then
    echo "  · aucun répertoire de migrations — rien à vérifier"
    exit 0
fi

# Référence de comparaison. `origin/<base>` en intégration continue, `<base>` en local.
reference=""
for candidat in "origin/$base" "$base"; do
    if git rev-parse --verify --quiet "$candidat" >/dev/null 2>&1; then
        reference="$candidat"
        break
    fi
done

if [[ -z "$reference" ]]; then
    echo "  ⚠ branche de base « $base » introuvable — la porte ne peut rien comparer."
    echo "    Attendu sur un dépôt neuf ou une branche orpheline ; anormal en intégration continue."
    exit 0
fi

echo "  référence : $reference"

echec=0
modifiees=()

# `git diff --name-status` distingue Ajout (A) de Modification (M), Renommage (R) et
# Suppression (D). Seul A est acceptable sur une migration.
while IFS=$'\t' read -r statut fichier reste; do
    [[ -z "${fichier:-}" ]] && continue
    case "$statut" in
        A) ;;                                   # nouvelle migration — normal
        M) modifiees+=("$fichier (modifié)"); echec=1 ;;
        D) modifiees+=("$fichier (supprimé)"); echec=1 ;;
        R*) modifiees+=("$fichier → ${reste:-?} (renommé)"); echec=1 ;;
        *) modifiees+=("$fichier ($statut)"); echec=1 ;;
    esac
# `git diff <ref> -- <chemin>` compare la référence à **l'arbre de travail**, pas seulement à
# HEAD. La nuance compte : en local, une migration modifiée mais non encore commitée doit être
# signalée maintenant — pas après le commit, quand la corriger demande de réécrire l'historique.
done < <(git diff --name-status "$reference" -- "$MIGRATIONS" 2>/dev/null || true)

if [[ $echec -ne 0 ]]; then
    echo >&2
    echo "P-02 ÉCHOUE — ${#modifiees[@]} migration(s) touchée(s) :" >&2
    for entree in "${modifiees[@]}"; do
        echo "  ✗ $entree" >&2
    done
    echo >&2
    echo "Une migration appliquée n'est JAMAIS modifiée (principe I(b))." >&2
    echo "Elle décrit un état que des bases portent déjà : la modifier crée une divergence" >&2
    echo "entre le dépôt et la production, invisible jusqu'à la création d'une base neuve." >&2
    echo "Créer une migration NOUVELLE qui corrige." >&2
    exit 1
fi

nombre=$(find "$MIGRATIONS" -maxdepth 1 -name '*.sql' | wc -l | tr -d ' ')
echo "P-02 ✓ — $nombre migration(s), aucune modifiée."
