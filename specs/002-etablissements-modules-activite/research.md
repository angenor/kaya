# Recherche — Cycle 002 · Établissements, modules d'activité et configuration héritée

**Phase 0 du plan** · 2026-07-31 · [plan.md](plan.md) · [spec.md](spec.md)

Quatorze décisions. Chacune porte sur un point que le cycle rencontrera de toute façon et qu'il
trancherait mal sous la pression de l'implémentation. Trois d'entre elles — R-01, R-03 et R-10 —
sont des **pièges de PostgreSQL sous sécurité au niveau ligne forcée** qui font échouer une
migration ou, pire, la font réussir sans rien écrire.

**Aucun numéro de version n'est proposé ici.** `docs/versions-gelees.md` fait foi ; les deux
paquets manquants sont signalés en R-13 sans version, comme le veut le principe XI.

---

## R-01 — Un référentiel global sous sécurité au niveau ligne forcée

**Question.** `module_activite`, `capacite` et `profil_stock` sont partagés par tous les tenants :
ils ne portent pas de `tenant_id`. Or la porte P-07 exige qu'aucune table n'échappe à une
politique, et le principe III exige `ENABLE` **et** `FORCE`. Comment concilier les deux ?

**Décision.** Trois tables de référentiel, chacune avec **deux politiques et un jeu de privilèges
asymétrique** :

```sql
ALTER TABLE etablissements.module_activite ENABLE ROW LEVEL SECURITY;
ALTER TABLE etablissements.module_activite FORCE  ROW LEVEL SECURITY;

-- Lecture ouverte : le référentiel est le même pour tous les clients.
CREATE POLICY lecture_universelle ON etablissements.module_activite
    FOR SELECT USING (true);

-- Écriture réservée au propriétaire — migrations, outillage d'édition, harnais de test.
CREATE POLICY administration_editeur ON etablissements.module_activite
    FOR ALL TO kaya_owner USING (true) WITH CHECK (true);

GRANT SELECT ON etablissements.module_activite TO kaya_app;   -- et rien d'autre
```

`kaya_app` est refusé **deux fois** : aucun privilège d'écriture, et aucune politique qui
l'autoriserait. Un `GRANT` accordé par erreur plus tard ne suffirait donc pas à ouvrir la table.

**Le piège que cette forme évite.** `FORCE` applique les politiques **au propriétaire des tables**.
Une migration qui crée la table, active `FORCE`, puis insère les cinq modules **échoue** — le
propriétaire n'a aucune politique d'écriture à ce moment-là. Deux parades, et l'ordre compte :

1. la politique `administration_editeur ... TO kaya_owner` ci-dessus ;
2. dans la migration, **insérer les valeurs avant** `ENABLE`/`FORCE` — ceinture et bretelles.

**Alternatives rejetées.**

| Alternative | Rejetée parce que |
|---|---|
| Référentiel dupliqué par tenant (`tenant_id` sur chaque ligne) | Multiplie cinq lignes par le nombre de clients, et rend impossible l'ajout d'une valeur « par configuration » (§14.3) : il faudrait l'écrire chez chacun |
| `ALTER TABLE ... NO FORCE` le temps de la migration | Un `NO FORCE` non rétabli laisse la table ouverte au propriétaire, et rien ne le signale |
| Table hors des schémas applicatifs, exclue de P-07 | Une exclusion par exception affaiblit la porte la plus structurante de l'isolation |

**Conséquence à écrire dans la migration.** L'exception est **nommée en commentaire**, comme l'a
été `tenant` au cycle 001 : « seule table dont la colonne comparée est sa propre clé ». Une
exception écrite est relisible ; une exception silencieuse devient un précédent.

---

## R-02 — Refuser une capacité non implémentée, de façon non contournable

**Question.** La porte P-06 exige que toute valeur autre que `STOCK`/`SIMPLE` soit refusée
explicitement. Une validation applicative se contourne par un script de maintenance, un import ou
un jeu de données. Où poser le refus ?

**Décision.** **Aux trois couches, avec trois rôles distincts** — c'est la même logique que la
longueur de texte du module doré, où le `CHECK` et la validation applicative coexistent.

| Couche | Rôle | Forme |
|---|---|---|
| Base | Dernier rempart, imparable | Clé étrangère composite + `CHECK` (ci-dessous) |
| Service | Message clair nommant la valeur | Variante d'erreur `CapaciteNonImplementee { code }` |
| Interface | Absence pure | La valeur n'est jamais proposée (principe VII) |

Le rempart de base est **déclaratif, sans déclencheur** :

```sql
-- Le référentiel porte l'état d'implémentation…
CREATE TABLE etablissements.capacite (
    code        TEXT PRIMARY KEY,
    implementee BOOLEAN NOT NULL,
    UNIQUE (code, implementee)          -- support de la clé étrangère composite
);

-- …et la déclaration de consommation le recopie, puis exige qu'il soit vrai.
CREATE TABLE etablissements.module_capacite (
    capacite_code        TEXT    NOT NULL,
    capacite_implementee BOOLEAN NOT NULL,
    FOREIGN KEY (capacite_code, capacite_implementee)
        REFERENCES etablissements.capacite (code, implementee),
    CHECK (capacite_implementee)
);
```

Déclarer `LIVRAISON` devient impossible : la seule ligne de référentiel qui la porte a
`implementee = false`, et le `CHECK` refuse. Le jour où une capacité est implémentée, un `UPDATE`
du référentiel l'ouvre — et la clé étrangère met à jour les lignes existantes en cascade si on le
demande explicitement.

**Alternatives rejetées.**

| Alternative | Rejetée parce que |
|---|---|
| Déclencheur `BEFORE INSERT` vérifiant le référentiel | Du code caché dans la base, invisible en lecture de schéma, et qui doit être maintenu en parallèle du référentiel |
| `CHECK (capacite_code = 'STOCK')` en dur | Ferait de l'ouverture d'une capacité une **migration**, alors que §14.4 en fait une écriture de configuration |
| Validation applicative seule | Contournable par tout chemin qui n'est pas le service — import, seed, script de reprise |

---

## R-03 — Le profil de stock est un référentiel, pas une contrainte de valeur

**Question.** Le cadrage §14.5 écrit « colonne `profil_stock ∈ {AUCUN, SIMPLE, VALORISE,
DETAILLE}` ». Un `CHECK ... IN (...)` suffirait-il ?

**Décision.** Non — **une quatrième table de référentiel**, `profil_stock`, avec la même colonne
`implementee` et le même motif de clé étrangère composite qu'en R-02.

**Rationale.** Le message de refus doit distinguer deux situations que l'utilisateur vit
différemment : « `VALORISE` existe mais n'est pas implémenté au MVP » et « `EXOTIQUE` n'existe
pas ». Un `CHECK` littéral ne peut produire que le second message pour les deux cas — il ne sait
pas ce qu'il refuse. La spécification l'exige explicitement (FR-033 : « un message nommant le
profil »).

Le profil `AUCUN` est **refusé comme les autres**, conformément à l'hypothèse 2 de la
spécification : une capacité non consommée ne se déclare pas. Son message de refus est distinct et
le dit — c'est le seul cas où le refus enseigne quelque chose à l'appelant plutôt que de constater
une absence de fonctionnalité.

**Alternative rejetée.** Un type énuméré PostgreSQL : ajouter une valeur à un `ENUM` est une
migration, et l'ordre des valeurs devient significatif sans qu'on l'ait voulu.

---

## R-04 — Stockage des paramètres de la chaîne d'héritage

**Question.** ETB-04 exige une résolution à quatre niveaux sur des paramètres hétérogènes —
durées, montants entiers, barèmes à paliers, booléens, chaînes. Colonnes typées, table par niveau,
ou table unique clé/valeur ?

**Décision.** **Une table unique**, `parametre_configuration`, avec une portée discriminée :

```
parametre_configuration (tenant_id, portee, portee_id, cle, valeur JSONB)
UNIQUE (tenant_id, portee, portee_id, cle)
portee ∈ {TENANT, ETABLISSEMENT, MODULE, POINT_DE_VENTE}
```

**Rationale.** Le registre des classes hors-ligne nomme déjà l'entité au singulier
(`parametre_configuration` — « toute valeur de la chaîne d'héritage »). Le récapitulatif de
`docs/user-stories-v1.md` compte une trentaine de paramètres, répartis sur huit cycles : une
colonne typée par paramètre imposerait **une migration par story**, sur une table que tous les
modules liront.

**Ce qui empêche la table de devenir un dépotoir** — le point qui décide de la qualité de cette
décision : un **catalogue** en table, `parametre_catalogue`, déclare chaque clé, son type de
valeur et le niveau le plus bas où elle peut être définie. Une écriture dont la clé n'y figure pas
est refusée. Le catalogue est global, en lecture seule pour les tenants, même motif qu'en R-01.

**La porte de cohérence documentaire.** Un test compare le catalogue au « Récapitulatif des
paramètres d'établissement » de `docs/user-stories-v1.md` : **toute clé du catalogue y figure**.
Le sens de la comparaison est asymétrique, exactement comme `classes_offline.rs` — une ligne du
récapitulatif sans clé au catalogue est normale (le paramètre relève d'un cycle futur), une clé
sans ligne est l'erreur à attraper. Le principe I·c devient ainsi vérifiable au lieu d'être
seulement écrit.

**Alternatives rejetées.**

| Alternative | Rejetée parce que |
|---|---|
| Quatre tables, une par niveau | La résolution ferait quatre requêtes ou une union à quatre branches ; ajouter le niveau « caisse » un jour exigerait une cinquième table et une réécriture du résolveur |
| Colonnes typées sur `etablissement` | Une migration par paramètre, sur la table la plus lue du produit |
| `JSONB` sans catalogue | Aucune validation possible, aucune découvrabilité, et le récapitulatif du principe I·c resterait décoratif |

---

## R-05 — Ce qui reste colonne, et ne devient pas paramètre

**Question.** Le récapitulatif de `docs/user-stories-v1.md` liste « classement », « commune »,
« fuseau horaire » et « devise » comme *paramètres d'établissement*. Doivent-ils vivre dans la
chaîne d'héritage ?

**Décision.** **Non — ils restent des colonnes de `etablissement`.**

**Rationale.** Le récapitulatif est l'**inventaire de ce qui est configurable** ; la chaîne
d'héritage est le **mécanisme des valeurs surchargeables par niveau**. Les deux ne se recouvrent
pas. La devise d'un établissement n'est pas héritée du tenant ni surchargeable par point de
vente : elle qualifie l'établissement, et la fiscalité, la caisse et la clôture la lisent sur
chaque montant. La porter en `JSONB` ferait passer par une résolution à quatre niveaux la valeur
la plus chaude du produit, sans qu'aucun niveau ne puisse jamais la surcharger.

**Écrit ici pour éviter la migration inverse.** Sans cette phrase, un cycle ultérieur lira le
récapitulatif, constatera que « devise » y figure, et la déplacera vers la table de paramètres —
là où l'entier d'unité mineure perdrait son typage.

**Règle de partage, opposable** : une valeur va dans la chaîne d'héritage **si et seulement si un
niveau inférieur peut légitimement la surcharger**. Sinon, c'est une colonne.

---

## R-06 — Le harnais des trois parcours et la détection des étapes dues

**Question.** FR-025 exige que l'intégration continue échoue lorsqu'une étape déclarée due devient
réalisable sans être branchée. Comment un test constate-t-il qu'un cycle ultérieur a livré une
étape ?

**Décision.** Chaque étape due déclare une **sentinelle observable**, nommée une par une :

```rust
Etape {
    nom: "vente_comptoir",
    cycle_du: "PDV — tranche T2",
    sentinelle: Sentinelle::Table { schema: "documents", table: "commande" },
    branchement: None,          // Some(fn) le jour où PDV la branche
}
```

À chaque exécution, le harnais interroge `information_schema.tables` et
`application::contrat_complet()`. Trois issues, et une seule est verte :

| Sentinelle | Branchement | Issue |
|---|---|---|
| absente | absent | **vert** — l'étape est due, son cycle n'est pas passé |
| présente | présent | **vert** — l'étape est exercée, son résultat est vérifié |
| présente | absent | **échec** — un cycle a livré l'étape sans la brancher |

**Trois exigences du § « Couverture des portes »**, tenues explicitement :

- le harnais **déclare en tête** ce qu'il inspecte (`information_schema` des schémas applicatifs,
  chemins du contrat complet) et ce qu'il n'inspecte pas (le comportement métier de l'étape, qui
  est l'affaire du cycle qui la livre) ;
- il **compte** les étapes exercées et les compare au total déclaré, et affiche les deux ;
- il **ne modifie jamais** ce qu'il inspecte — la création du service fictif se fait dans une
  transaction annulée en fin de parcours, et le catalogue de sentinelles est lu, jamais écrit.

**Sentinelles nommées une par une, jamais par motif.** Un motif — « toute table du schéma
`caisse` » — passerait au vert sur une table sans rapport et raterait la bonne. Même règle que
`TABLES_EXCLUES` de `classes_offline.rs`.

**Alternative rejetée.** Un fichier de suivi que chaque cycle mettrait à jour à la main : c'est
exactement la revue humaine que FR-025 exclut, et le cycle 001 a montré que quatre portes vertes
défectueuses n'ont été trouvées par aucune relecture.

---

## R-07 — Le service fictif minimal : créé par le test, sous quel rôle

**Question.** Le référentiel des modules est en écriture réservée au propriétaire (R-01). Le
parcours d'agnosticité doit y insérer un module fictif. Comment, sans laisser de trace ?

**Décision.** Le harnais utilise **`commun::pool_owner()`** — le rôle propriétaire, déjà exposé
par le support de tests du cycle 001 — et travaille dans une **transaction annulée** : le module
fictif n'est jamais commité.

Deux conséquences vérifiées par des tests distincts :

- une vérification échoue si le code du service fictif apparaît dans une table après exécution du
  jeu de seeds (FR-027) ;
- le parcours ne dépend d'aucun autre et peut s'exécuter en parallèle des deux autres : ses
  écritures n'existent que dans sa transaction.

**Code retenu** : `MODULE_FICTIF_TEST`. Explicite au point d'être impossible à confondre avec une
valeur de production, et repérable par une recherche textuelle.

**Alternative rejetée.** Un module fictif permanent au référentiel, marqué « réservé aux tests » :
il finirait proposé à l'activation par la console éditeur le jour où celle-ci lira le référentiel,
et un drapeau « ne pas afficher » est précisément le grisé que le principe VII interdit.

---

## R-08 — Migration additive sur une table déjà peuplée et sous `FORCE`

**Question.** ETB-01 ajoute sept colonnes à `etablissement`, dont plusieurs `NOT NULL`. Deux
établissements sont déjà seedés. Comment les remplir ?

**Décision.** **`ADD COLUMN ... NOT NULL DEFAULT ...`, puis `DROP DEFAULT` si la valeur par défaut
n'a pas de sens permanent.**

**Le piège, et il est grave.** La parade naturelle — ajouter la colonne en `NULL`, puis
`UPDATE ... SET juridiction = 'CI'`, puis `SET NOT NULL` — **ne fonctionne pas** : la migration
s'exécute sous `kaya_owner`, `etablissement` est en `FORCE ROW LEVEL SECURITY`, et la politique
compare à `current_setting('app.current_tenant', true)`, qui vaut `NULL` hors requête applicative.
L'`UPDATE` ne touche **aucune ligne** — et **ne lève aucune erreur**. La migration réussit, la
colonne reste vide, et le `SET NOT NULL` échoue plus loin avec un message qui ne dit rien de la
cause.

`ADD COLUMN ... DEFAULT` est du **DDL** : il remplit les lignes existantes sans passer par une
politique de sécurité. C'est la seule forme qui traverse `FORCE`.

**Règle générale, à porter au patron.** *Aucune migration n'écrit de données par `INSERT` ou
`UPDATE` sur une table en `FORCE ROW LEVEL SECURITY`.* Ce qui doit être écrit passe par le DDL
(`DEFAULT`) ou par la mécanique de seeds, qui s'exécute sous le rôle applicatif **et pose le
tenant courant**. Cette règle vaut pour tous les cycles suivants ; à ce titre elle est reportée
dans `docs/module-dore.md`.

**Valeurs retenues pour les deux établissements existants** : juridiction `CI`, classement
`NON_CLASSE`, commune `Abengourou` pour Deloria. « Résidence Test » reçoit les mêmes valeurs
structurelles ; sa commune est portée par les seeds du cycle, non par le `DEFAULT`.

---

## R-09 — La résolution de configuration : forme du trait et du résultat

**Question.** ETB-04 exige un trait propre au crate `etablissements`, consommé par tous les
modules. Quelle signature ?

**Décision.** Un trait **dyn-compatible** (`async_trait`, comme `OutboxWriter`), rendant une
valeur **et son origine**, et distinguant l'absence de la valeur vide :

```rust
pub enum Portee { Tenant, Etablissement, Module, PointDeVente }

pub struct Cible {                       // le point d'où l'on résout
    pub tenant_id: Uuid,
    pub etablissement_id: Option<Uuid>,
    pub module_code: Option<String>,
    pub point_de_vente_id: Option<Uuid>,
}

pub struct ValeurResolue {
    pub valeur: serde_json::Value,
    pub origine: Portee,                 // le niveau qui a fourni la valeur
}

#[async_trait::async_trait]
pub trait ResolveurConfiguration: Send + Sync {
    async fn resoudre(&self, cible: &Cible, cle: &str)
        -> Result<Option<ValeurResolue>, ErreurConfiguration>;

    async fn resoudre_tout(&self, cible: &Cible)
        -> Result<BTreeMap<String, ValeurResolue>, ErreurConfiguration>;
}
```

Trois choix, chacun pour une raison précise :

- **`Option<ValeurResolue>`, jamais une valeur par défaut** (FR-048). Un défaut rendu par le
  résolveur serait un paramètre en dur dans le code, ce que le principe I·c interdit — déguisé en
  commodité.
- **`origine` obligatoire, non optionnelle.** L'écran `G1` doit distinguer « hérité du tenant » de
  « surchargé ici », et un champ optionnel serait ignoré par le premier appelant pressé.
- **`resoudre_tout`** existe pour l'écran : résoudre trente paramètres un par un ferait trente
  allers-retours. Une seule requête, une seule descente de chaîne.

**Chaîne écourtée.** `Cible` porte des `Option` : un établissement sans service ni point de vente
résout sur deux niveaux, sans niveau inventé (FR-050). La requête ne joint que les niveaux
présents.

**Alternative rejetée.** Un résolveur générique typé `resoudre<T: DeserializeOwned>` : le type
attendu vivrait alors chez l'appelant, et deux modules pourraient lire la même clé avec deux types
différents. Le catalogue (R-04) porte le type ; la conversion se fait au bord, après validation.

---

## R-10 — Surcharge inerte plutôt que suppression

**Question.** FR-051 : une surcharge portée par un service désactivé devient inerte sans être
supprimée. Comment, sans que la requête de résolution devienne une jointure inter-schémas ?

**Décision.** La résolution **filtre sur l'état d'activation** au moment de la descente de chaîne,
par une jointure **interne au schéma `etablissements`** — `etablissement_module` y vit, tout comme
`parametre_configuration`. Aucune frontière de module n'est franchie, la porte P-04 n'est pas
concernée.

**Le point qu'on écrirait mal** : supprimer les surcharges à la désactivation. Ce serait une perte
de données silencieuse et irréversible, sous couvert de nettoyage. La réactivation d'un service
doit restituer exactement l'état antérieur (FR-015) — ce que seule la conservation permet.

---

## R-11 — Identifiants fournis par le client, y compris en classe C

**Question.** Toutes les entités de ce cycle sont de classe C : jamais écrites hors ligne. L'UUID
v7 côté client, motif du module doré, est-il encore justifié ?

**Décision.** **Oui, sans exception.** Le cadrage §11.5.1 l'exige « sur toute écriture, classes A
et D comprises », et l'argument tient aussi en ligne : un double-clic sur « Créer l'établissement »
ou un renvoi de formulaire après un délai réseau produirait deux établissements. `ON CONFLICT (id)
DO NOTHING ... RETURNING` distingue `201` de `200` sans second aller-retour.

**Conséquence sur les codes de retour** : `200` sur rejeu, jamais `409` — même règle qu'au module
doré, pour la même raison.

---

## R-12 — Le rattachement de caisse d'un point de vente

**Question.** ETB-03 donne `caisse_rattachee` au point de vente. `socle/caisse` n'a aucune table
avant le cycle CAI.

**Décision.** Colonne `caisse_id UUID NULL`, **sans clé étrangère**, avec un commentaire qui dit
pourquoi. C'est exactement le motif du module doré sur `auteur_compte_id` : l'intégrité
référentielle entre modules passe par un trait exposé, jamais par la base — et ce serait vrai même
si la table existait déjà.

Le cycle CAI ajoutera la vérification par trait, non une clé étrangère.

---

## R-13 — Tester « un service inactif est absent de l'interface »

**Question.** SC-005 exige zéro occurrence d'un service inactif dans l'**interface rendue**. Le
harnais front du cycle 001 ne monte aucun composant : `vitest` y est configuré en environnement
`node`, et ni `@vue/test-utils` ni environnement DOM ne sont installés.

**Décision.** **Deux niveaux, dont un seul appelle de nouvelles dépendances.**

1. **Test unitaire de la sélection** — la fonction qui, à partir des services activés, produit la
   liste des sections visibles est **pure**. Elle se teste sans DOM, sans nouvelle dépendance, et
   c'est elle qui porte la règle.
2. **Test de rendu** — monte `G1` avec un établissement à service unique et vérifie qu'aucun
   libellé ni code des quatre autres services n'apparaît dans le HTML produit. Exige un
   utilitaire de montage de composants Vue et un environnement DOM.

**Écart au gel, signalé sans être tranché.** `docs/versions-gelees.md` §3.2 ne contient ni
utilitaire de montage Vue ni environnement DOM. **Aucune version n'est proposée ici** : les deux
paquets sont vérifiés sur le registre officiel et épinglés exactement **au moment de leur ajout**
(tâche de `/speckit-tasks`, URL citée), puis portés au gel à la revue mensuelle du 2026-08-31 —
même procédure que les six crates ajoutés par le cycle 001.

**Si l'ajout devait être refusé**, le niveau 1 seul reste opposable et SC-005 se vérifie alors sur
la fonction de sélection plutôt que sur le rendu. C'est une couverture moindre, à consigner
explicitement plutôt qu'à laisser croire acquise.

---

## R-14 — Chargement paresseux par module, sans extension Nuxt

**Question.** Le principe VII exige un chargement paresseux par module : « un serveur de salle ne
télécharge pas le code du back-office ». `nuxt.config.ts` réserve déjà `app/modules/` aux modules
métier. Comment y rattacher les pages ?

**Décision.** **Import dynamique, pas d'extension Nuxt.** La page reste dans `app/pages/`, réduite
à quelques lignes ; tout le contenu métier vit dans `app/modules/etablissements/` et est chargé
par `defineAsyncComponent(() => import(...))`. Vite produit un fragment séparé par module, et le
découpage est vérifiable sur la sortie de construction.

**Alternative rejetée.** Une extension Nuxt par module métier, enregistrant ses pages par
`extendPages` : c'est la forme correcte à dix modules, et de la complexité prématurée à un seul.
La bascule se fera au cycle où un troisième module aura des pages — elle ne coûte rien de plus
alors, et elle coûterait un cycle de mise au point aujourd'hui.

**Conséquence pour `PlatformAdapter`** : `G1` n'appelle **aucune** capacité native. Le choix de
fichier du logo passe par un `<input type="file">` standard, servi par la vue web sous Tauri
comme sur le web. Aucune extension de l'adaptateur n'est nécessaire, et la porte P-15 n'a rien de
nouveau à surveiller.

---

## Ce que la recherche n'a pas tranché

| Point | Pourquoi il reste ouvert | Échéance |
|---|---|---|
| Versions des deux paquets de test front (R-13) | Le principe XI interdit de proposer un numéro de mémoire ; la vérification se fait au moment de l'ajout, URL citée | `/speckit-tasks` |
| Format et taille maximale du logo | Sans effet sur le modèle de données ; la référence en base est une clé d'objet quelle que soit la réponse | Implémentation |
| Jeu de valeurs de `politique_impression` | Défini par le cycle IMP (tranche T2). Ce cycle pose la clé au catalogue, sans valeur | Cycle IMP |
| **B-02** — traitement fiscal de la taxe de nuitée sur passage et demi-journée | Décision produit, hors de portée d'un cycle technique. Aucune valeur en dur n'est posée en attendant | Atelier fiscaliste |
