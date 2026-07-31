#!/usr/bin/env bash
#
# **Porte P-04** — aucune requête ne joint deux schémas de modules différents.
#
# ═════════════════════════════════════════════════════════════════════════════════════════════
#  LA LIMITE DE CETTE PORTE, EN TÊTE DE FICHIER PLUTÔT QU'ENFOUIE DANS UN COMMENTAIRE
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
#  **C'est une HEURISTIQUE, et elle est assumée comme telle** (`plan.md`, Complexity Tracking,
#  écart 5).
#
#  Elle repère deux préfixes de schéma distincts dans une même requête. Elle NE FAIT PAS d'analyse
#  syntaxique du SQL. Concrètement :
#
#    ✓ elle attrape le cas courant — un `JOIN` écrit entre deux schémas dans une même requête ;
#    ✗ elle ne voit pas une jointure construite dynamiquement, ni cachée derrière une vue, ni
#      répartie entre une CTE et son usage ;
#    ✗ elle peut signaler une requête qui *nomme* deux schémas sans les joindre — un `UNION`, ou
#      deux sous-requêtes indépendantes.
#
#  Une analyse complète du SQL serait disproportionnée à ce stade. **La revue mensuelle couvre le
#  reste** (constitution, § Revue).
#
#  Cette limite est écrite ici, et non dans un commentaire au milieu du script, pour qu'un
#  développeur qui voit la porte au vert sache exactement ce qu'elle a vérifié — et ce qu'elle
#  n'a pas vérifié.
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
# # Pourquoi la règle existe
#
# Un schéma par module, et les lectures inter-modules par un **trait exposé** (principe II). C'est
# ce qui rend un module extractible en service sans réécriture : une jointure inter-schémas est
# une dépendance que rien ne déclare et que personne ne voit venir.
#
# L'alternative existe déjà — `EstablishmentDirectory` est posé, à vide, précisément pour que le
# premier `JOIN` inter-schémas ne soit pas écrit « juste cette fois ».

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

# Schémas de modules. En ajouter un ici est **obligatoire** à la création de son module : un
# schéma inconnu de cette liste échappe entièrement à la porte.
SCHEMAS=(etablissements synchronisation fiscalite comptes caisse documents pilotage editeur metriques stocks hebergement restauration bar pressing)

echo "── P-04 — jointures entre schémas de modules ─────────────────────────────────"
echo "  heuristique assumée — voir l'en-tête de ce script"

echec=0

analyser() {
    local fichier="$1"
    local contenu
    # Commentaires retirés : ce dépôt documente abondamment, et un commentaire qui cite deux
    # schémas n'est pas une requête.
    contenu="$(sed -e 's|--.*$||' -e 's|//.*$||' "$fichier" | tr '\n' ' ')"

    # Découpage grossier en requêtes, sur les points-virgules et les délimiteurs de macro.
    local requete
    while IFS= read -r requete; do
        [[ -z "${requete// /}" ]] && continue
        # Une requête doit au moins ressembler à du SQL.
        if ! grep -qiE '\b(select|insert|update|delete)\b' <<<"$requete"; then
            continue
        fi

        local trouves=()
        for schema in "${SCHEMAS[@]}"; do
            if grep -qE "\b${schema}\." <<<"$requete"; then
                trouves+=("$schema")
            fi
        done

        if [[ ${#trouves[@]} -gt 1 ]]; then
            echo "  ✗ ${fichier} — une requête nomme ${#trouves[@]} schémas : ${trouves[*]}" >&2
            echo "      ${requete:0:140}…" >&2
            echec=1
        fi
    done < <(tr ';' '\n' <<<"$contenu")
}

# Migrations : les fichiers de définition de schéma nomment forcément plusieurs schémas
# (`REFERENCES etablissements.tenant` depuis `fiscalite`). Ce sont des CLÉS ÉTRANGÈRES, pas des
# requêtes — et le principe II interdit les jointures de lecture, pas l'intégrité déclarative.
# Elles sont donc hors périmètre, et c'est délibéré.
while IFS= read -r fichier; do
    analyser "$fichier"
done < <(find backend/api backend/crates backend/tests -name '*.rs' -not -path '*/target/*' 2>/dev/null | sort)

if [[ $echec -ne 0 ]]; then
    echo >&2
    echo "P-04 ÉCHOUE — une requête joint deux schémas de modules." >&2
    echo "Les lectures inter-modules passent par un TRAIT exposé (principe II) :" >&2
    echo "  specs/001-socle-technique-monorepo/contracts/traits-exposes.md" >&2
    echo "C'est ce qui rend un module extractible en service sans réécriture." >&2
    exit 1
fi

echo "P-04 ✓ — aucune requête ne nomme deux schémas de modules."
