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
#
# **Cette liste n'est plus tenue par la seule discipline** (cycle 004) : le contrôle de complétude
# ci-dessous relit les `CREATE SCHEMA` des migrations et fait échouer la porte sur tout schéma
# réellement créé qui manquerait ici. C'est l'exigence de la section « Couverture des portes » de
# la constitution — *un test négatif prouve qu'une porte sait échouer, il ne prouve pas qu'elle
# regarde tout*.
SCHEMAS=(etablissements synchronisation fiscalite comptes caisse documents pilotage editeur metriques stocks hebergement restauration bar pressing)

# ═════════════════════════════════════════════════════════════════════════════════════════════
#  PAIRES SENSIBLES — les couples dont la jointure serait ÉCRITE SPONTANÉMENT
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
# La détection ci-dessous est **générique** : elle attrape n'importe quel couple de schémas nommés
# dans une même requête, et l'a toujours fait. Ces déclarations n'ajoutent donc **aucune détection
# nouvelle** — elles ajoutent la **preuve que la cible n'est pas vide**, qui est une exigence
# distincte de la section « Couverture des portes » de la constitution :
#
#   > *Un test négatif prouve qu'une porte sait échouer ; il ne prouve pas qu'elle regarde tout.
#   > Une porte dont la cible est vide passe toujours.*
#
# Une paire déclarée ici fait **échouer la porte** si l'un de ses deux schémas n'a **aucune**
# requête dans le périmètre inspecté. Le mode d'échec visé est précis : si `hebergement` cessait
# d'avoir des requêtes — un crate renommé, un chemin de recherche cassé —, P-04 resterait verte en
# ne regardant rien, et son vert serait indiscernable d'un vert mérité.
#
# **`comptes hebergement` est la paire de ce cycle**, et c'est la première fois que deux schémas
# se parlent sur le **chemin chaud** : un séjour affiche toujours le nom de son client. C'est la
# jointure que tout le monde écrirait. Elle n'existe pas, et trois mécanismes le garantissent —
# aucune clé étrangère (`sejour.client_id` est un UUID nu), le trait `AnnuaireClients`, et cette
# porte-ci. Le sens inverse est le plus dangereux : l'historique des séjours d'un client est servi
# **depuis `hebergement`**, jamais depuis `comptes`, sans quoi ce serait une jointure inter-schémas
# **et** une arête `socle/ → verticales/` — P-04 et P-03 d'un coup.
PAIRES_SENSIBLES=("comptes hebergement")

echo "── P-04 — jointures entre schémas de modules ─────────────────────────────────"
echo "  heuristique assumée — voir l'en-tête de ce script"

echec=0

# ═════════════════════════════════════════════════════════════════════════════════════════════
#  1. COMPLÉTUDE DU PÉRIMÈTRE — tout schéma créé par une migration est-il inspecté ?
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
# Sans ce contrôle, créer `hebergement` sans l'ajouter à `SCHEMAS` aurait laissé la porte verte en
# n'inspectant rien de ce cycle. Le symptôme d'une porte à cible vide est exactement celui d'une
# porte satisfaite.
while IFS= read -r schema_reel; do
    [[ -z "$schema_reel" ]] && continue
    connu=0
    for schema in "${SCHEMAS[@]}"; do
        [[ "$schema" == "$schema_reel" ]] && connu=1 && break
    done
    if [[ $connu -eq 0 ]]; then
        echo "  ✗ le schéma « ${schema_reel} » est créé par une migration et ABSENT de SCHEMAS" >&2
        echo "      il échapperait entièrement à cette porte" >&2
        echec=1
    fi
done < <(grep -rhoiE 'CREATE SCHEMA (IF NOT EXISTS )?[a-z_]+' backend/migrations/*.sql 2>/dev/null \
         | awk '{print tolower($NF)}' | sort -u)

# Décompte des requêtes nommant chaque schéma — tableaux parallèles à SCHEMAS, pour rester
# compatible avec bash 3.2 (macOS), qui n'a pas de tableau associatif.
compte_par_schema=()
for _ in "${SCHEMAS[@]}"; do compte_par_schema+=(0); done
fichiers_analyses=0
requetes_analysees=0

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
        requetes_analysees=$((requetes_analysees + 1))

        local trouves=()
        local i=0
        for schema in "${SCHEMAS[@]}"; do
            if grep -qE "\b${schema}\." <<<"$requete"; then
                trouves+=("$schema")
                compte_par_schema[$i]=$(( ${compte_par_schema[$i]} + 1 ))
            fi
            i=$((i + 1))
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
    fichiers_analyses=$((fichiers_analyses + 1))
    analyser "$fichier"
done < <(find backend/api backend/crates backend/tests -name '*.rs' -not -path '*/target/*' 2>/dev/null | sort)

# ═════════════════════════════════════════════════════════════════════════════════════════════
#  2. PÉRIMÈTRE RÉELLEMENT INSPECTÉ — le décompte, pas seulement le verdict
# ═════════════════════════════════════════════════════════════════════════════════════════════
echo
echo "  périmètre inspecté : ${fichiers_analyses} fichier(s), ${requetes_analysees} requête(s)"
i=0
for schema in "${SCHEMAS[@]}"; do
    nombre="${compte_par_schema[$i]}"
    if [[ "$nombre" -gt 0 ]]; then
        printf '    %-16s %4d requête(s)\n' "$schema" "$nombre"
    else
        printf '    %-16s    · aucune requête — schéma sans code, la porte ne peut rien y trouver\n' "$schema"
    fi
    i=$((i + 1))
done

# ═════════════════════════════════════════════════════════════════════════════════════════════
#  3. LES PAIRES SENSIBLES ONT-ELLES DEUX CÔTÉS NON VIDES ?
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
# Un schéma sans requête ne peut produire aucune jointure : la porte le laisserait passer sans
# rien avoir regardé. Pour les couples déclarés ci-dessus, ce silence est un échec.
compte_du_schema() {
    local cherche="$1" i=0
    for schema in "${SCHEMAS[@]}"; do
        if [[ "$schema" == "$cherche" ]]; then
            echo "${compte_par_schema[$i]}"
            return 0
        fi
        i=$((i + 1))
    done
    echo "0"
}

echo
echo "  paires sensibles — les deux côtés doivent être non vides :"
for paire in "${PAIRES_SENSIBLES[@]}"; do
    gauche="${paire%% *}"
    droite="${paire##* }"
    n_gauche="$(compte_du_schema "$gauche")"
    n_droite="$(compte_du_schema "$droite")"

    if [[ "$n_gauche" -eq 0 || "$n_droite" -eq 0 ]]; then
        printf '    ✗ %-12s × %-12s  %d / %d requête(s)\n' "$gauche" "$droite" "$n_gauche" "$n_droite" >&2
        echo "        un côté est VIDE : la porte ne peut trouver aucune jointure entre ces deux" >&2
        echo "        schémas, et son vert ne dirait rien de plus que « je n'ai rien regardé »." >&2
        echec=1
    else
        printf '    ✓ %-12s × %-12s  %d / %d requête(s) — la paire est réellement inspectée\n' \
            "$gauche" "$droite" "$n_gauche" "$n_droite"
    fi
done

# Une porte qui n'analyse rien passe toujours au vert. Le cas s'est produit au cycle 001 sur
# P-08, paramétrée sur un contrat vide (module doré, « Une porte peut mentir »).
if [[ $requetes_analysees -eq 0 ]]; then
    echo >&2
    echo "P-04 ÉCHOUE — AUCUNE requête analysée. La porte n'a pas de cible : son vert ne dirait" >&2
    echo "rien. Vérifier le chemin de recherche des fichiers." >&2
    exit 1
fi

if [[ $echec -ne 0 ]]; then
    echo >&2
    echo "P-04 ÉCHOUE — une requête joint deux schémas de modules." >&2
    echo "Les lectures inter-modules passent par un TRAIT exposé (principe II) :" >&2
    echo "  specs/001-socle-technique-monorepo/contracts/traits-exposes.md" >&2
    echo "C'est ce qui rend un module extractible en service sans réécriture." >&2
    exit 1
fi

echo "P-04 ✓ — aucune requête ne nomme deux schémas de modules."
