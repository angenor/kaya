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
#
# # Ce que la porte ne regarde PAS, et pourquoi (corrigé au cycle 004)
#
# **Seuls les `.sql` à la racine de `backend/migrations/` sont des migrations.** Le sous-répertoire
# `seeds/` n'en contient aucune : le principe I(b) sépare les deux, précisément parce qu'une
# migration n'est jamais rejouée et qu'un seed l'est constamment. Un seed **doit** pouvoir changer.
#
# Jusqu'au cycle 004, le diff portait sur tout le répertoire : la mise à jour de
# `backend/migrations/seeds/README.md` faisait échouer la porte, avec un message demandant de
# « créer une migration nouvelle qui corrige » — pour un fichier de documentation. Le décompte final
# de ce script, lui, était déjà limité à `-maxdepth 1` : les deux extrémités se contredisaient.
#
# Le nombre de fichiers réellement comparés est affiché, sans quoi restreindre une cible reviendrait
# à la vider sans que rien ne le dise (constitution, § « Couverture des portes »).

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
comparees=0

# `git diff --name-status` distingue Ajout (A) de Modification (M), Renommage (R) et
# Suppression (D). Seul A est acceptable sur une migration.
while IFS=$'\t' read -r statut fichier reste; do
    [[ -z "${fichier:-}" ]] && continue

    # Une migration est un `.sql` **à la racine** du répertoire. Tout le reste — `seeds/`, la
    # documentation — n'en est pas une, et le principe I(b) veut qu'il puisse changer.
    relatif="${fichier#"$MIGRATIONS"/}"
    [[ "$relatif" == *.sql && "$relatif" != */* ]] || continue
    comparees=$((comparees + 1))

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

if [[ "$nombre" -eq 0 ]]; then
    echo >&2
    echo "P-02 ÉCHOUE — aucune migration trouvée dans « $MIGRATIONS »." >&2
    echo "Une porte dont la cible est vide passe toujours au vert : celle-ci refuse de le faire." >&2
    exit 1
fi

echo "  $comparees fichier(s) .sql comparé(s) à la référence (seeds et documentation exclus)"
echo "P-02 ✓ — $nombre migration(s) au total, aucune modifiée."
