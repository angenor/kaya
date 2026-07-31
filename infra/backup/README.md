# Sauvegarde et restauration — Kaya

**Ce document est écrit pour quelqu'un qui n'a pas construit le système.** C'est la seule façon de
savoir s'il est complet : une procédure rédigée pour son auteur omet toujours ce qu'il sait déjà,
et ce qu'il sait déjà est exactement ce qui manquera à 3 h du matin, six mois plus tard, quand
quelqu'un d'autre devra restaurer.

Suivez-le **dans l'ordre**, sans sauter d'étape.

---

## 1. Le point à comprendre avant tout le reste

Il y a **deux copies** de chaque sauvegarde, et elles ne servent pas à la même chose.

| Copie | Où | Sert à | Ne sert **pas** à |
|---|---|---|---|
| **Tiers** | Stockage objet d'un fournisseur externe, **hôte distinct**, verrouillage d'objet | Se relever d'une **compromission** ou d'un rançongiciel | — |
| **Locale** | Garage, sur le VPS de production | Restaurer vite un incident ordinaire | Se relever d'une compromission |

**Garage tourne sur le même serveur que la base.** Un attaquant qui obtient le serveur obtient les
deux ; un rançongiciel qui chiffre le disque chiffre la sauvegarde avec. La copie locale est un
confort, pas une garantie.

Une sauvegarde présente **uniquement** dans Garage **ne satisfait pas FR-060**.

---

## 2. Ce qu'il vous faut avant de commencer

| Élément | Où le trouver | Attention |
|---|---|---|
| **Clé privée `age`** | Coffre de l'éditeur — **jamais sur le serveur de production** | Sans elle, aucune sauvegarde n'est lisible. C'est le seul élément irremplaçable. |
| Accès au stockage tiers | Coffre de l'éditeur | Lecture suffit pour restaurer |
| `age`, `aws` CLI, `pg_restore` | Poste de restauration | Versions : `age` ≥ 1.0, `postgresql-client` de la **même version majeure** que la production |
| Un serveur PostgreSQL **vierge** | À provisionner | Jamais celui de production |

> **La clé privée n'est pas sur le serveur.** C'est délibéré : `sauvegarder.sh` chiffre avec la
> clé **publique**. Un serveur compromis ne peut donc pas relire les sauvegardes qu'il a
> produites — mais cela signifie aussi que **perdre la clé privée, c'est perdre toutes les
> sauvegardes**. Elle est conservée en deux exemplaires, dans deux lieux distincts.

---

## 3. Variables d'environnement

```sh
# Base de production (pour la sauvegarde)
export PGHOST=... PGPORT=5432 PGDATABASE=kaya PGUSER=kaya_owner PGPASSWORD=...

# Chiffrement
export KAYA_BACKUP_AGE_RECIPIENT="age1..."        # clé PUBLIQUE — sauvegarde
export KAYA_BACKUP_AGE_IDENTITY=/chemin/cle.txt   # clé PRIVÉE  — restauration seulement

# Stockage tiers — hôte distinct, verrouillage d'objet activé sur le compartiment
export KAYA_BACKUP_S3_TIERS_BUCKET=kaya-sauvegardes
export KAYA_BACKUP_S3_TIERS_ENDPOINT=https://...

# Garage — copie de travail, facultative
export KAYA_BACKUP_S3_LOCAL_BUCKET=kaya-sauvegardes-locales
export S3_ENDPOINT=http://localhost:3900
```

---

## 4. Restaurer

```sh
# 1. Lister les sauvegardes disponibles
aws s3 ls "s3://${KAYA_BACKUP_S3_TIERS_BUCKET}/quotidien/" \
    --endpoint-url "$KAYA_BACKUP_S3_TIERS_ENDPOINT"

# 2. Restaurer dans une base NEUVE — jamais par-dessus la production
infra/backup/restaurer.sh 20260731T031500Z kaya_restauration

# 3. Amorcer les rôles
psql -d kaya_restauration -f infra/postgres/init/00-kaya-owner.sql

# 4. Appliquer les migrations
cd backend/api && cargo sqlx migrate run --source ../migrations
```

**Pourquoi jamais par-dessus la production** : restaurer sur la base vive écrase tout ce qui a été
produit depuis la sauvegarde. On restaure à côté, on vérifie, **puis** on bascule. `restaurer.sh`
refuse d'ailleurs de viser la base source.

**Pourquoi les migrations après la restauration** : le dump est pris `--no-owner --no-privileges`.
Les rôles et les droits viennent des migrations, seule source de vérité du schéma (principe I(b)).
Restaurer des droits depuis un dump les figerait à leur état du jour de la sauvegarde.

---

## 5. Vérifier que la restauration a réussi

Ne pas se fier à l'absence d'erreur. Vérifier ces cinq points :

```sql
-- 1. Les six tables sont là
SELECT table_schema || '.' || table_name
FROM information_schema.tables
WHERE table_schema IN ('etablissements', 'synchronisation', 'fiscalite')
ORDER BY 1;

-- 2. La sécurité au niveau ligne est ACTIVE ET FORCÉE partout
SELECT relname, relrowsecurity, relforcerowsecurity
FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE n.nspname IN ('etablissements','synchronisation','fiscalite') AND c.relkind = 'r';

-- 3. Le grand livre est complet — comparer au décompte noté AVANT la sauvegarde
SELECT COUNT(*) FROM synchronisation.evenement_outbox;

-- 4. Le déclencheur d'immuabilité est en place
SELECT tgname FROM pg_trigger WHERE tgname = 'evenement_outbox_immuable';

-- 5. Les migrations sont toutes inscrites
SELECT version, description FROM kaya_migrations._migrations_appliquees ORDER BY version;
```

Puis, depuis le dépôt, la vérification qui vaut les cinq précédentes :

```sh
DATABASE_URL=postgres://kaya_owner:...@.../kaya_restauration \
  cargo test -p kaya-backend --test rls_catalogue
```

Le point 2 est celui qu'on oublie : `pg_restore` recrée les tables, mais une erreur d'option peut
laisser la sécurité au niveau ligne désactivée. La base fonctionnerait, et **tous les clients
verraient les données de tous les autres**.

---

## 6. Journal des exercices de restauration

**Une sauvegarde jamais restaurée n'est pas une sauvegarde.** Exercice à refaire à chaque revue
mensuelle, et à consigner ici — même quand il se passe bien, surtout la durée.

| Date | Volume | Durée | Exécuté par | Écarts constatés |
|---|---|---|---|---|
| 2026-07-31 | *(base de développement, ~200 Kio)* | *(non chronométré)* | Cycle 001 | **Exercice complet NON RÉALISÉ.** Voir ci-dessous. |

### État réel au 2026-07-31 — à lire, pas à contourner

**Les deux scripts sont écrits et la procédure est complète, mais l'exercice de restauration en
environnement vierge (T053) n'a pas été exécuté.** Il lui manque trois éléments qui n'existent pas
encore et qui ne relèvent pas du code :

1. un **fournisseur de stockage tiers** choisi et provisionné — R-13 arrête l'invariant (hôte
   distinct, verrouillage d'objet, rétention verrouillée) et laisse explicitement le nom du
   fournisseur à trancher ;
2. une **paire de clés `age`** de production, déposée au coffre en deux exemplaires ;
3. un **serveur vierge** pour restaurer.

Ce qui a été vérifié en revanche :

- les deux scripts sont écrits, exécutables, et refusent les cas dangereux (dump suspect sous
  1 Kio, restauration par-dessus la production, variable requise absente) ;
- la procédure ci-dessus est rédigée pour un tiers, avec ses vérifications.

**Ce qui reste dû** : le premier exercice chronométré, en environnement vierge, **suivi par
quelqu'un qui n'a pas écrit le système**. C'est la seule façon de savoir si ce document est
complet. Tant qu'il n'a pas eu lieu, **FR-060 n'est pas satisfaite** et SC-006 reste ouvert.

---

## 7. Supervision

Voir `infra/supervision/README.md` — l'alerte au-delà de 2 minutes d'indisponibilité (FR-057) est
**hébergée hors du serveur surveillé**, faute de quoi elle ne prouve rien : un serveur mort
n'envoie pas d'alerte disant qu'il est mort.
