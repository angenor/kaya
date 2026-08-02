# Phase 1 — Modèle de données

**Cycle 004 — HEB** · Unités louables, formules de location et moteur de disponibilité
**Date** : 2026-08-02 · **Spec** : [spec.md](./spec.md) · **Recherche** : [research.md](./research.md)

Six migrations, huit tables, cinq types d'événements outbox. Toutes les décisions techniques
viennent de `research.md` ; ce document les met en schéma.

---

## 1. Vue d'ensemble

```
schéma hebergement — 8 tables
├── categorie                    C   ── un groupe d'unités homogènes
│   ├── temps_remise_en_etat     C   ── par catégorie ET par formule
│   ├── unite                    C   ── une chambre, un logement, une salle
│   └── formule                  C   ── ce qu'on vend sur cette catégorie
│       ├── bareme_palier        C   ── paliers du passage
│       ├── plage_demi_journee   C   ── plages fixes de la demi-journée
│       └── prestation_incluse   —   ── PROVISION HEB-09, table vide
└── occupation                   B   ── l'attribution d'une unité sur un intervalle
                                         ↑ LA table du cycle. EXCLUDE USING gist.
```

**Aucune clé étrangère ne sort du schéma `hebergement`.** `etablissement_id` et `tenant_id` sont
des UUID sans `REFERENCES` — une clé vers `etablissements.etablissement` serait une clé
inter-schémas, interdite par le principe II et la porte P-04. La cohérence est tenue par les
traits exposés (`EstablishmentDirectory`, `RegistreModules`), comme au cycle 003 pour
`comptes.permission`.

---

## 2. Migration `0021_schema_hebergement.sql` — le schéma et ses privilèges

```sql
CREATE SCHEMA IF NOT EXISTS hebergement;

GRANT USAGE ON SCHEMA hebergement TO kaya_app;
```

**Une migration dédiée**, comme `0014_schema_comptes.sql` au cycle 003. Motif inscrit dans cette
migration-là : un `CREATE SCHEMA` glissé dans une migration ancienne produirait un écart entre les
schémas déclarés et les schémas réels, que P-04 fait échouer.

L'extension `btree_gist` n'est **pas** réinstallée : `0001_roles_et_schemas.sql:93` l'a fait, et
elle est globale à la base.

---

## 3. Migration `0024_referentiel_hebergement.sql` — six tables de classe C

### 3.1 `categorie`

| Colonne | Type | Notes |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | UUID v7 posé par le client (principe VI) |
| `tenant_id` | `UUID NOT NULL` | Sans `REFERENCES` — clé inter-schémas interdite |
| `etablissement_id` | `UUID NOT NULL` | Idem |
| `nom` | `TEXT NOT NULL` | « Standard », « Classique », « Salle de réunion » |
| `capacite_accueil` | `SMALLINT NOT NULL CHECK (capacite_accueil > 0)` | Nombre de personnes |
| `cree_le` | `TIMESTAMPTZ NOT NULL DEFAULT now()` | Horodatage d'autorité |
| `modifie_le` | `TIMESTAMPTZ NOT NULL DEFAULT now()` | |

```sql
CONSTRAINT categorie_nom_unique UNIQUE (etablissement_id, nom)
```

**`capacite_accueil` n'est pas `capacite`.** Le lexique réserve « capacité » au transverse (stock,
livraison, fidélité) et écrit qu'il « n'apparaît jamais » à l'utilisateur. Nommer cette colonne
`capacite` créerait deux sens pour un mot déjà chargé, dans deux schémas voisins.

### 3.2 `temps_remise_en_etat`

Le temps de remise en état est **par catégorie ET par formule** (HEB-01 : `categorie {…,
temps_remise_en_etat_par_formule}`). C'est donc une table, pas une colonne.

| Colonne | Type | Notes |
|---|---|---|
| `categorie_id` | `UUID NOT NULL REFERENCES hebergement.categorie (id) ON DELETE CASCADE` | |
| `famille_formule` | `TEXT NOT NULL CHECK (famille_formule IN ('NUITEE','PASSAGE','DEMI_JOURNEE','MENSUEL'))` | |
| `duree_minutes` | `INTEGER NOT NULL CHECK (duree_minutes >= 0)` | 30 · 120 · 60 aux seeds |
| `tenant_id` | `UUID NOT NULL` | Porté pour la politique RLS |

```sql
PRIMARY KEY (categorie_id, famille_formule)
```

**`duree_minutes >= 0` et non `> 0`** : une catégorie peut légitimement n'avoir aucun battement
(une salle de réunion sans ménage entre deux réunions). Zéro est une valeur, pas une absence.

### 3.3 `unite`

| Colonne | Type | Notes |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | |
| `tenant_id`, `etablissement_id` | `UUID NOT NULL` | |
| `categorie_id` | `UUID NOT NULL REFERENCES hebergement.categorie (id)` | |
| `code` | `TEXT NOT NULL` | « A1 », « B3 », « SALLE-1 » |
| `etage` | `SMALLINT NULL` | Nul pour une salle en rez-de-chaussée non numéroté |
| `statut_menage` | `TEXT NOT NULL DEFAULT 'propre' CHECK (statut_menage IN ('a_nettoyer','propre','maintenance'))` | **Colonne seule — aucun endpoint, HEB-06** |
| `cree_le`, `modifie_le` | `TIMESTAMPTZ NOT NULL DEFAULT now()` | |

```sql
CONSTRAINT unite_code_unique UNIQUE (etablissement_id, code)
```

> **Aucune colonne `statut_occupation`.** Elle est **dérivée** des occupations (R-10, cadrage
> §11.4). L'inscrire en table rendrait possible de la poser à la main, ce que le cadrage désigne
> comme la cause des doubles attributions. Une relecture ultérieure qui la chercherait doit
> trouver ce paragraphe plutôt que le vide.

### 3.4 `formule`

| Colonne | Type | Notes |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | |
| `tenant_id`, `etablissement_id` | `UUID NOT NULL` | |
| `categorie_id` | `UUID NOT NULL REFERENCES hebergement.categorie (id)` | **La formule est attachée à la catégorie, jamais au type d'établissement** (FR-017, FR-019) |
| `famille` | `TEXT NOT NULL CHECK (famille IN ('NUITEE','PASSAGE','DEMI_JOURNEE','MENSUEL'))` | Toute autre valeur **refusée explicitement** (FR-022) |
| `prix_mineur` | `BIGINT NOT NULL CHECK (prix_mineur >= 0)` | **Entier d'unité mineure** (P-10). Prix d'appel : la nuit, le mois, la plage. Pour `PASSAGE`, c'est le premier palier — la table de barème fait foi |
| `duree_min_minutes` | `INTEGER NULL CHECK (duree_min_minutes > 0)` | |
| `duree_max_minutes` | `INTEGER NULL CHECK (duree_max_minutes > 0)` | |
| `heure_arrivee_standard` | `TIME NULL` | 14 h pour la nuitée |
| `heure_depart_standard` | `TIME NULL` | 12 h pour la nuitée |
| `jours_autorises` | `SMALLINT[] NULL` | 1–7, nul = tous |
| `assujettie_taxe_nuitee` | `BOOLEAN NOT NULL` | **Éditable** — c'est le « moyen facultatif d'ajouter la taxe » quand une commune l'impose |
| `regle_conversion_taxe` | `TEXT NULL CHECK (regle_conversion_taxe IN ('aucune','une_nuitee_par_occupation','au_prorata','seuil_horaire'))` | `NULL` permis **seulement** sur une formule non assujettie (R-14). `une_nuitee_par_occupation` = 500 F pour 3 nuits ; `au_prorata` = 500 F × 3 |
| `prix_heure_supplementaire_mineur` | `BIGINT NULL CHECK (… >= 0)` | Renseigné pour `PASSAGE` seul |
| `cree_le`, `modifie_le` | `TIMESTAMPTZ NOT NULL DEFAULT now()` | |

```sql
-- FR-021 : une catégorie ne porte pas deux formules de la même famille.
CONSTRAINT formule_famille_unique UNIQUE (categorie_id, famille),

-- FR-020 : une durée maximale inférieure à la minimale est inexploitable.
CONSTRAINT formule_durees_coherentes
    CHECK (duree_min_minutes IS NULL OR duree_max_minutes IS NULL
           OR duree_max_minutes >= duree_min_minutes),

-- Le prix d'heure supplémentaire n'a de sens que sur le passage.
CONSTRAINT formule_heure_sup_reservee_au_passage
    CHECK (prix_heure_supplementaire_mineur IS NULL OR famille = 'PASSAGE'),

-- R-14 : une formule assujettie SANS règle de conversion est une incohérence, pas un état
-- d'attente. Cette contrainte est ce qui supprime le besoin d'un troisième état d'écran —
-- « paramétrage fiscal en attente » n'existe ni à la maquette G2 ni au lexique.
CONSTRAINT formule_regle_fiscale_coherente
    CHECK (NOT assujettie_taxe_nuitee OR regle_conversion_taxe IS NOT NULL)
```

> **Ce que la base ne peut pas garantir, et où c'est tenu.** Qu'une formule `PASSAGE` porte au
> moins un palier, et qu'une `DEMI_JOURNEE` porte au moins une plage (FR-025, FR-033), ne
> s'exprime pas en contrainte de table — la dépendance va de l'enfant au parent. C'est le
> **service** qui le valide, dans la transaction de création, et
> `backend/tests/hebergement_referentiel.rs` qui le vérifie. Écrit ici pour qu'on ne cherche pas
> une contrainte absente.

### 3.5 `bareme_palier`

| Colonne | Type | Notes |
|---|---|---|
| `formule_id` | `UUID NOT NULL REFERENCES hebergement.formule (id) ON DELETE CASCADE` | |
| `duree_minutes` | `INTEGER NOT NULL CHECK (duree_minutes > 0)` | **`> 0`** — FR-025 refuse un palier de durée nulle |
| `prix_mineur` | `BIGINT NOT NULL CHECK (prix_mineur >= 0)` | |
| `tenant_id` | `UUID NOT NULL` | Pour la politique RLS |

```sql
-- FR-025 : deux paliers de même durée sont impossibles, donc l'ordre est total.
PRIMARY KEY (formule_id, duree_minutes)
```

**L'unicité de la durée est la clé primaire**, pas une contrainte ajoutée : c'est ce qui rend
« un barème aux paliers désordonnés » impossible à constituer plutôt qu'à corriger (R-11). La
lecture trie par `duree_minutes`.

### 3.6 `plage_demi_journee`

| Colonne | Type | Notes |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | |
| `formule_id` | `UUID NOT NULL REFERENCES hebergement.formule (id) ON DELETE CASCADE` | |
| `heure_debut` | `TIME NOT NULL` | **Heure murale locale** (R-13) |
| `heure_fin` | `TIME NOT NULL` | |
| `libelle_cle` | `TEXT NOT NULL` | Clé i18n — « matin », « après-midi » |
| `tenant_id` | `UUID NOT NULL` | |

```sql
CONSTRAINT plage_bornes CHECK (heure_fin > heure_debut),
CONSTRAINT plage_unique UNIQUE (formule_id, heure_debut, heure_fin)
```

**`TIME` et non `TIMESTAMPTZ`** : « 8 h – 12 h » est une règle d'exploitation qui vaut tous les
jours, y compris ceux qui n'existent pas encore. La stocker en instant imposerait une ligne par
jour (R-13). La conversion en instant se fait au serveur, avec le fuseau de l'établissement.

**`heure_fin > heure_debut`** interdit une plage qui traverse minuit. Assumé : une demi-journée
qui franchit minuit n'est pas une demi-journée. Une formule de nuit se modélise en `PASSAGE` ou en
`NUITEE`.

### 3.7 Le patron RLS, appliqué aux six tables

Identique partout, tel que le module doré le fixe :

```sql
ALTER TABLE hebergement.<table> ENABLE ROW LEVEL SECURITY;
ALTER TABLE hebergement.<table> FORCE  ROW LEVEL SECURITY;

CREATE POLICY isolation_tenant ON hebergement.<table>
    USING      (tenant_id = current_setting('app.current_tenant', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.current_tenant', true)::uuid);
```

Les trois éléments, aucun optionnel : `FORCE` (sans lui le propriétaire reste hors politique),
`WITH CHECK` (sans lui un tenant peut **insérer** chez un autre — la fuite qui n'apparaît dans
aucune lecture), et le second argument `true` de `current_setting` (sans lui une transaction sans
contexte lève une erreur au lieu de ne rien voir).

**C'est pourquoi `tenant_id` est porté par les tables filles** — `temps_remise_en_etat`,
`bareme_palier`, `plage_demi_journee` — alors qu'il serait dérivable du parent. Une politique RLS
qui devrait joindre le parent pour trouver le tenant serait à la fois plus lente et plus fragile.

### 3.8 Privilèges

```sql
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.categorie             TO kaya_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.temps_remise_en_etat  TO kaya_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.unite                 TO kaya_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.formule               TO kaya_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.bareme_palier         TO kaya_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON hebergement.plage_demi_journee    TO kaya_app;
```

Classe C — référentiel éditable. Les privilèges disent la classe (module doré) ; ceux
d'`occupation` diffèrent, voir §4.

---

## 4. Migration `0025_occupation.sql` — **la table du cycle**

C'est la seule migration du projet dont une erreur ne se rattrape pas. La contrainte d'exclusion
est posée **à la création**, jamais après : ajoutée sur une table peuplée, elle échoue sur les
données existantes, et il faudrait alors choisir entre corriger l'historique et renoncer à la
garantie (R-02).

| Colonne | Type | Notes |
|---|---|---|
| `id` | `UUID PRIMARY KEY` | UUID v7 client |
| `tenant_id`, `etablissement_id` | `UUID NOT NULL` | |
| `unite_id` | `UUID NOT NULL REFERENCES hebergement.unite (id)` | |
| `formule_id` | `UUID NOT NULL REFERENCES hebergement.formule (id)` | |
| **`periode`** | **`TSTZRANGE NOT NULL`** | **Remise en état COMPRISE** (R-04). C'est cette colonne que la contrainte protège |
| `debut_client` | `TIMESTAMPTZ NOT NULL` | Borne commerciale — ce que le client connaît |
| `fin_client` | `TIMESTAMPTZ NOT NULL` | Idem. La note se calcule là-dessus |
| `statut` | `TEXT NOT NULL DEFAULT 'active' CHECK (statut IN ('active','liberee'))` | |
| `libere_le` | `TIMESTAMPTZ NULL` | Horodatage d'autorité de la libération |
| `cree_le` | `TIMESTAMPTZ NOT NULL DEFAULT now()` | **Horodatage d'autorité** — jamais l'horloge du terminal |

### 4.1 Les contraintes, dans l'ordre où elles comptent

```sql
-- ═══ LA contrainte du cycle ═══
-- Le chevauchement devient IMPOSSIBLE AU NIVEAU DE LA BASE, pas seulement dans le code.
-- `unite_id WITH =` exige `btree_gist` : GiST ne sait pas indexer l'égalité sur un UUID sans
-- l'extension. Elle est installée par 0001_roles_et_schemas.sql:93.
CONSTRAINT occupation_sans_chevauchement
    EXCLUDE USING gist (
        unite_id WITH =,
        periode  WITH &&
    ),

-- Le SEUL contournement possible de la contrainte ci-dessus (R-06).
-- `&&` est FAUX dès qu'un intervalle est vide : `[14h, 14h)` passerait l'exclusion ET
-- n'empêcherait aucune autre occupation. Une ligne fantôme qui occupe sans bloquer.
CONSTRAINT occupation_periode_non_vide
    CHECK (NOT isempty(periode)),

-- Verrouille la forme `[)`. Avec `[]`, deux occupations contiguës deviendraient chevauchantes,
-- et le comportement du produit changerait selon la forme employée par l'appelant.
CONSTRAINT occupation_periode_semi_ouverte
    CHECK (lower_inc(periode) AND NOT upper_inc(periode)),

-- Les bornes commerciales sont dans la période d'indisponibilité, jamais hors d'elle.
-- La remise en état ALLONGE la période ; elle ne la déplace pas.
CONSTRAINT occupation_bornes_client_coherentes
    CHECK (fin_client > debut_client
           AND lower(periode) <= debut_client
           AND upper(periode) >= fin_client),

-- Une occupation libérée porte son horodatage, une active n'en porte pas.
CONSTRAINT occupation_liberation_coherente
    CHECK ((statut = 'liberee') = (libere_le IS NOT NULL))
```

### 4.2 Privilèges — **pas de `DELETE`**

```sql
GRANT SELECT, INSERT, UPDATE ON hebergement.occupation TO kaya_app;
```

Une occupation ne se supprime pas : elle se **libère**, ce qui est un `UPDATE` de sa période et de
son statut. Accorder `DELETE` permettrait d'effacer la trace d'une chambre occupée — et le
classement en B deviendrait faux sans que rien ne le signale (module doré, § « Les privilèges
disent la classe »).

### 4.3 RLS

Même patron que §3.7.

### 4.4 Index

**Aucun index supplémentaire.** L'index GiST créé par la contrainte d'exclusion sert exactement la
requête la plus fréquente du produit — chercher les occupations d'une unité qui chevauchent un
intervalle. Un index B-tree sur `(unite_id, cree_le)` serait ajouté sans mesure ; le principe X
l'interdit tant qu'aucun besoin ne l'appelle.

---

## 5. Migration `0026_provision_prestation_incluse.sql` — HEB-09, table seule

| Colonne | Type |
|---|---|
| `id` | `UUID PRIMARY KEY` |
| `tenant_id` | `UUID NOT NULL` |
| `formule_id` | `UUID NOT NULL REFERENCES hebergement.formule (id) ON DELETE CASCADE` |
| `type_prestation` | `TEXT NOT NULL` |
| `quantite` | `NUMERIC NOT NULL CHECK (quantite > 0)` | **`NUMERIC`, jamais entier** (P-10) — un petit-déjeuner se compte à l'unité, une prestation de blanchisserie au kilo |
| `valeur_unitaire_plafond_mineur` | `BIGINT NOT NULL CHECK (… >= 0)` | Entier d'unité mineure |

```sql
COMMENT ON TABLE hebergement.prestation_incluse IS
    'PROVISION HEB-09 — table seulement. Aucun endpoint, aucune logique, aucun écran. La
     fonctionnalité arrive en incrément 2. Classe C au registre §7.1.';
```

**RLS activée et forcée comme les autres** — P-07 ne fait pas d'exception pour une provision — et
**aucun privilège accordé à `kaya_app`** : rien ne l'écrit ni ne la lit à ce cycle.
`backend/tests/provisions_sans_logique.rs` vérifie déjà ce couple pour les provisions des cycles
précédents ; il gagne une cible.

**`quantite` en `NUMERIC` est le point qui ne se rattrape pas.** Passer d'entier à décimal après
mise en production imposerait de migrer toutes les lignes de tous les clients. La colonne est
posée juste alors même qu'aucun code ne la lit.

---

## 6. Migration `0022_permissions_hebergement.sql` — les premières permissions de module

```sql
INSERT INTO comptes.permission (code, module_code, libelle_cle, ordre) VALUES
    ('heb.offre.lire',              'HEBERGEMENT', 'comptes.permissions.heb.offre.lire',              180),
    ('heb.offre.gerer',             'HEBERGEMENT', 'comptes.permissions.heb.offre.gerer',             190),
    ('heb.disponibilite.consulter', 'HEBERGEMENT', 'comptes.permissions.heb.disponibilite.consulter', 200),
    ('heb.unite.attribuer',         'HEBERGEMENT', 'comptes.permissions.heb.unite.attribuer',         210),
    ('heb.unite.liberer',           'HEBERGEMENT', 'comptes.permissions.heb.unite.liberer',           220);
```

**`module_code` non nul pour la première fois du produit.** La migration `0016` du cycle 003
l'annonce : « `module_code` restera donc `NULL` **jusqu'au cycle HEB, qui apportera
`heb.unite.attribuer`** ». Ce cycle honore cette phrase à la lettre.

**Toujours sans clé étrangère** vers `etablissements.module_activite` — ce serait une clé
inter-schémas (P-04). La cohérence est tenue par le test existant qui lit le référentiel des
modules à travers le trait `RegistreModules` ; il gagne cinq cibles, et c'est la première fois
qu'il vérifie autre chose que `NULL`.

Attribution aux rôles :

| Rôle | Permissions |
|---|---|
| `proprietaire`, `gerant` | les cinq |
| `receptionniste` | `heb.offre.lire`, `heb.disponibilite.consulter`, `heb.unite.attribuer`, `heb.unite.liberer` — **pas `heb.offre.gerer`** : Yao attribue des chambres, il ne fixe pas les tarifs |
| autres | aucune |

**L'`INSERT` est possible en migration** parce que `comptes.permission` est un référentiel
**global** et que la politique `administration_editeur … TO kaya_owner` posée au cycle 003
l'autorise. Sans elle, l'insertion réussirait **en n'écrivant rien** — le piège du module doré.

---

## 7. Migration `0023_parametres_hebergement.sql` — trois clés au catalogue

```sql
INSERT INTO etablissements.parametre_catalogue
    (cle, type_valeur, portee_la_plus_basse, story, libelle_cle, description_cle)
VALUES
    ('heure_arrivee_standard',       'TEXTE',  'ETABLISSEMENT', 'HEB-03', …),
    ('heure_depart_standard',        'TEXTE',  'ETABLISSEMENT', 'HEB-03', …),
    ('seuil_bascule_nuitee_minutes', 'ENTIER', 'ETABLISSEMENT', 'HEB-04', …);
```

**Ce qui ne va PAS au catalogue, et pourquoi** (R-16) :

- le **temps de remise en état** — il varie par catégorie *et* par formule, ce n'est pas un
  scalaire d'établissement. Il vit dans `hebergement.temps_remise_en_etat` (§3.2) ;
- les **plages de demi-journée** — le registre §7.1 les classe comme référentiel. Elles vivent
  dans `hebergement.plage_demi_journee` (§3.6) ;
- le **barème de passage** — même motif, table `bareme_palier`.

Le récapitulatif des paramètres de `docs/user-stories-v1.md` les liste comme **valeurs par défaut
Deloria**, ce que les seeds honorent en les posant sur les catégories et formules. Le
récapitulatif est mis à jour **dans le même changement** (principe I·c) avec les trois clés
nouvelles.

---

## 8. Les cinq types d'événements outbox

| Type | Émis par | Charge utile |
|---|---|---|
| `heb.occupation.attribuee` | `OccupationService::attribuer` | `unite_id`, `formule_id`, `debut_client`, `fin_client`, borne haute de `periode` |
| `heb.occupation.liberee` | `OccupationService::liberer` | `occupation_id`, `libere_le`, nouvelle borne haute |
| `heb.formule.creee` | `FormuleService::creer` | `formule_id`, `famille`, `prix_mineur`, `devise` |
| `heb.formule.modifiee` | `FormuleService::modifier` | `formule_id`, champs changés |
| `heb.categorie.tarif_modifie` | `FormuleService::modifier_prix` | `formule_id`, `prix_mineur` avant et après, `devise` |

**Toujours dans la transaction de l'écriture**, garanti par la signature du trait :

```rust
async fn ecrire(&self, tx: &mut sqlx::PgTransaction<'_>, evenement: EvenementAEcrire)
    -> Result<(), ErreurOutbox>;
```

`OutboxWriter::ecrire` **prend la transaction et n'en ouvre jamais une** (module doré, couche 4).
Écrire l'événement ailleurs demanderait de fabriquer une seconde transaction et de la passer
explicitement — ce qui se voit en revue.

**Jamais sur rejeu.** Une seconde soumission du même UUID v7 ne produit aucun événement : le grand
livre porte les transitions d'état, pas les tentatives réseau du terminal.

**Nommage monétaire (P-10)** : les charges utiles portent `prix_mineur` (entier) et `devise` au
même niveau — jamais `prix`, `montant` ni `total` nus, que le contrôle statique refuse.

**Décompte P-05** : le total passe de **22 à 27** types déclarés au modèle de données.
`backend/tests/couverture_portes.rs` compare ce total aux types réellement testés et échoue sur
tout écart — c'est là que se perd un type introduit sans test.

---

## 9. Classes hors-ligne — déjà déclarées, à honorer

Les six entités sont **déjà au registre** `docs/registre-classes-offline.md` §7.1 et §7.2, écrites
au 2026-07-30 avant qu'aucune table n'existe. Elles sont **honorées, pas réécrites** — comme les
neuf entités du cycle 003.

| Entité | Classe | Branche | Registre |
|---|---|---|---|
| `categorie` | **C** | C2 — référentiel | §7.1 |
| `unite` | **C** | C2 — référentiel | §7.1 |
| `formule` | **C** | C2 — référentiel fiscal | §7.1 |
| `bareme_palier` | **C** | C2 — référentiel tarifaire | §7.1 |
| Plages de demi-journée | **C** | C2 — référentiel | §7.1 |
| `occupation` | **B** | B3 — ressource unique, contrainte d'exclusion GiST | §7.2 |
| Intervalle de remise en état | **B** | B3 — intégré à l'intervalle | §7.2 |
| `unite.statut_occupation` | **dérivé** | — | §7.2 |
| `unite.statut_menage` | **A** | A4 — dernier-écrit-gagne, seul cas | §7.2 |
| `prestation_incluse` | **C** | C2 — référentiel attaché à la formule | §7.1 (ligne HEB-09) |

**Deux tables que le registre ne nomme pas encore**, à ajouter au §7.1 dans le même changement :

- **`temps_remise_en_etat`** — le registre le mentionne comme attribut de `categorie` (« temps de
  remise en état par formule ») ; devenu table, il doit se déclarer pour lui-même. Classe **C**,
  branche C2, sur le régime de sa catégorie. Précédent exact : `profil_stock` au cycle 002.
- **`plage_demi_journee`** — le registre écrit « Plages de demi-journée », sans nom de table. La
  ligne est honorée, le nom précisé.

`backend/tests/classes_offline.rs` compare **table → registre** et fait échouer le build sur toute
table non déclarée. Son tableau `SCHEMAS_APPLICATIFS` gagne `hebergement` — **sans quoi les huit
tables du cycle échapperaient au balayage**, exactement le trou que le cycle 003 a trouvé sur le
schéma `comptes`.

Entrée au **journal §13** du registre dans le même changement, version **1.2.0**.

---

## 10. Ce que le modèle ne porte pas, et pourquoi

| Absent | Motif |
|---|---|
| `ressource_reservable` au socle | Abstraction spéculative à un seul implémenteur (R-09). Le socle ne gagne **aucune** notion d'unité — c'est P-03 qui le vérifie |
| `unite.statut_occupation` | Dérivé, jamais posé à la main (R-10) |
| `calendrier_tarifaire` | HEB-07, P1 — le principe X interdit de le poser d'avance |
| `contrat_location`, `caution`, `charge_locative`, `etat_des_lieux` | HEB-08, incrément 3 |
| Table de séjour, de client, de note | SEJ et FIS — ce cycle calcule, il ne facture pas (R-12) |
| Index B-tree sur `occupation` | L'index GiST de la contrainte sert la requête (§4.4) |
