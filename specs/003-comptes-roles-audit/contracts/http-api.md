# Contrat HTTP — cycle 003 (CPT)

**Dix-neuf opérations.** Le contrat OpenAPI est **produit par le code** (annotations utoipa) : ce
document dit ce que le code doit annoncer, il ne le remplace pas. Après toute modification de
handler : `scripts/ci/generer-client.sh`, puis commit du client (porte **P-01**).

**Total du contrat après ce cycle : 40 opérations** (21 existantes + 19).

---

## Règles reprises du module doré, couche 5

- Le **verbe et le chemin** viennent de l'attribut de routage Actix, **jamais** répétés dans
  `#[utoipa::path]`. Les écrire deux fois laisse le contrat annoncer une adresse que le serveur ne
  sert pas.
- Montage par `service(...)`, **jamais** `route(...)` : `utoipa-actix-web` ne collecte que le
  premier. Un endpoint monté par `route(...)` serait servi sans figurer au contrat — donc absent
  du client généré et **invisible pour la porte P-08**.
- **Ordre de montage** : le plus spécifique d'abord. `/session/actives/{session_id}` avant
  `/session/actives` avant `/session` ; `/comptes/{compte_id}/roles/{role_code}` avant
  `/comptes/{compte_id}/roles` avant `/comptes`.
- **`operationId` unique** — porte **P-01b**. Deux opérations homonymes produisent un client
  TypeScript invalide que P-01 ne détecte pas.
- **Aucun détail interne ne franchit la frontière** : ni message PostgreSQL, ni nom de table, ni
  trace. Le détail part dans les journaux, corrélé par identifiant de requête.
- **`200` sur rejeu, pas `409`**, et le corps rendu est la ligne **telle qu'elle est en base** :
  le serveur fait foi en conflit.

---

## Les dix-neuf opérations

### Session — CPT-01

| # | Verbe | Chemin | `operationId` | Auth | Permission |
|---|---|---|---|---|---|
| 1 | `POST` | `/api/v1/session` | `session_ouvrir` | **publique** | — |
| 2 | `POST` | `/api/v1/session/rafraichir` | `session_rafraichir` | **publique** | — |
| 3 | `DELETE` | `/api/v1/session` | `session_fermer` | jeton | — (soi) |
| 4 | `GET` | `/api/v1/session/moi` | `session_moi` | jeton | — (soi) |
| 5 | `GET` | `/api/v1/session/actives` | `session_lister_actives` | jeton | — (soi) |
| 6 | `DELETE` | `/api/v1/session/actives/{session_id}` | `session_revoquer` | jeton | `cpt.session.revoquer` **ou** soi |

**1 · `session_ouvrir`** — corps : `{ identifiant, mot_de_passe, etablissement_id? }`.

Réponse `200` : `{ acces, expire_dans_s, rafraichissement, compte, permissions[], etablissements[] }`.

- `permissions[]` est **l'union** des permissions de tous les rôles portés sur l'établissement
  actif (FR-017). Le front la lit **ici**, jamais en décodant le jeton (research R-06).
- `etablissements[]` porte les établissements accessibles ; le sélecteur permanent est **ETB-06,
  hors périmètre** (research R-07).
- Sans `etablissement_id`, le premier accessible par ordre stable devient actif.

**Réponse `401` : un seul code, `identifiants_invalides`.** Jamais `compte_inconnu`, jamais
`mot_de_passe_invalide`, jamais `compte_desactive` — FR-012. Et le **temps de réponse** est du
même ordre dans tous les cas (research R-02) : c'est la moitié de l'exigence que le code seul ne
tient pas.

**Réponse `422` : `methode_non_implementee`** quand le compte est réglé sur `OTP_SMS` (FR-008).
Refus **nommé**, jamais un repli silencieux sur le mot de passe.

**2 · `session_rafraichir`** — corps : `{ rafraichissement, etablissement_id? }`.
Le jeton consommé **ne se réemploie pas** (FR-010, rotation). Un jeton révoqué, inconnu ou déjà
consommé rend `401 session_invalide`. Les permissions sont **recalculées** à cette occasion : un
rôle retiré prend effet ici (hypothèse 5 de la spec).

**5 · `session_lister_actives`** — les sessions du compte appelant, avec libellé d'appareil,
première ouverture et dernière activité. Reconstruit depuis Redis : si Redis a été vidé, la liste
est vide et tout le monde s'est reconnecté (research R-01).

**6 · `session_revoquer`** — `204`. Effet au **refus du rafraîchissement suivant**. Émet
`session.revoquee` et une entrée d'audit. Révoquer sa propre session ne demande aucune permission.

---

### Personnes — CPT-00

| # | Verbe | Chemin | `operationId` | Permission |
|---|---|---|---|---|
| 7 | `POST` | `/api/v1/personnes` | `personne_creer` | `cpt.compte.gerer` |
| 8 | `GET` | `/api/v1/personnes/{personne_id}` | `personne_lire` | `cpt.compte.lire` |
| 9 | `PUT` | `/api/v1/personnes/{personne_id}` | `personne_modifier` | `cpt.compte.gerer` |

Corps de création : `{ id, nom, prenoms?, telephone?, email?, horodatage_client? }` — `id` est un
**UUID v7 généré côté client** (principe VI), ce qui rend le rejeu inoffensif : `201` puis `200`,
`200`.

**`type_piece` et `numero_piece` ne sont ni acceptés ni rendus.** Les colonnes existent
(data-model §1) ; leur alimentation relève de SEJ-01 et leur rétention de TRX-06. Le test de
provision vérifie qu'aucun point d'entrée de ce cycle ne les écrit.

**Aucune liste de personnes.** La recherche de fiches client est **SEJ-01**. Exposer une liste ici
donnerait au produit un annuaire d'identités civiles avant qu'il ait la politique de rétention qui
va avec.

---

### Comptes et rôles — CPT-01, CPT-02

| # | Verbe | Chemin | `operationId` | Permission |
|---|---|---|---|---|
| 10 | `POST` | `/api/v1/comptes` | `compte_creer` | `cpt.compte.gerer` |
| 11 | `GET` | `/api/v1/comptes` | `compte_lister` | `cpt.compte.lire` |
| 12 | `GET` | `/api/v1/comptes/{compte_id}` | `compte_lire` | `cpt.compte.lire` |
| 13 | `PUT` | `/api/v1/comptes/{compte_id}/etat` | `compte_changer_etat` | `cpt.compte.gerer` |
| 14 | `PUT` | `/api/v1/comptes/{compte_id}/mot-de-passe` | `compte_changer_mot_de_passe` | `cpt.compte.gerer` **ou** soi |
| 15 | `POST` | `/api/v1/comptes/{compte_id}/roles` | `compte_attribuer_role` | `cpt.role.attribuer` |
| 16 | `DELETE` | `/api/v1/comptes/{compte_id}/roles/{role_code}` | `compte_retirer_role` | `cpt.role.attribuer` |

**10 · `compte_creer`** — `{ id, personne_id, identifiant_telephone?, identifiant_email?,
mot_de_passe }`. Au moins un identifiant, sinon `422 identifiant_absent`.

Un identifiant déjà employé rend `422 identifiant_refuse` — **le message ne dit pas qu'il
existe déjà** (edge case de la spec) ; la tentative part au journal applicatif.

**Le condensat n'est jamais rendu**, sur aucune réponse, sur aucun chemin. Le repository expose
deux lectures distinctes pour que la structure d'affichage ne le porte même pas (data-model §3).

**11 · `compte_lister`** — écran `G3`. Rend, par compte : identité, état, **et les rôles portés
avec leur établissement**. Filtres : `etablissement_id`, `actif`, `role_code`.

**13 · `compte_changer_etat`** — `{ actif }`. Émet `compte.desactive` ou `compte.reactive`, plus
une entrée d'audit de type `suppression` (la désactivation **est** la suppression au sens de la
taxonomie — rien ne se supprime jamais, FR-014).

**14 · `compte_changer_mot_de_passe`** — un compte agissant sur lui-même fournit son mot de passe
actuel ; un compte habilité ne le fournit pas. Émet `compte.mot_de_passe_change` — **dont la
charge utile ne porte ni le secret ni son condensat**. Les autres sessions du compte sont
révoquées.

**15 · `compte_attribuer_role`** — `{ id, role_code, etablissement_id? }`. `etablissement_id` est
**obligatoire** pour un rôle de portée `ETABLISSEMENT`, **interdit** pour `admin_editeur`
(`422 portee_incompatible`). L'établissement est vérifié via `EstablishmentDirectory` →
`404 etablissement_inconnu`, jamais une violation de contrainte.

**16 · `compte_retirer_role`** — `etablissement_id` en paramètre de requête. Refus
`409 derniere_habilitation` si le retrait laisserait l'établissement sans aucun compte habilité à
attribuer les rôles (FR-023). C'est le seul refus « métier » du cycle, et il est irréversible sans
l'éditeur — d'où un code propre plutôt qu'un `403`.

**15 et 16 sont de classe C.** L'interface refuse **avant l'appel** quand le terminal est hors
ligne, et le dit (module doré, septième couche, point 6). Aucune mise en file « au cas où ».

---

### Référentiels — CPT-02

| # | Verbe | Chemin | `operationId` | Permission |
|---|---|---|---|---|
| 17 | `GET` | `/api/v1/referentiels/roles` | `referentiel_roles` | jeton |
| 18 | `GET` | `/api/v1/referentiels/permissions` | `referentiel_permissions` | jeton |

Les deux rendent la **même chose aux deux tenants** — ce sont des référentiels globaux. Le test
d'isolation P-08 l'affirme **explicitement**, comme pour les quatre référentiels d'ETB-02b : sans
cette assertion, un référentiel global et une fuite inter-tenants se ressemblent.

`libelle_cle` porte une **clé i18n**, jamais un libellé : une chaîne stockée en base échapperait à
la porte P-16.

---

### Journal d'audit — CPT-04

| # | Verbe | Chemin | `operationId` | Permission |
|---|---|---|---|---|
| 19 | `GET` | `/api/v1/journal-audit` | `journal_audit_lister` | `cpt.audit.consulter` |

Filtres combinables (FR-037) : `auteur_compte_id`, `etablissement_id`, `type_action`, `depuis`,
`jusqu_a`. Pagination par curseur sur `(cree_le DESC, id DESC)`.

Chaque entrée rend : type d'action, **auteur dénormalisé** (identifiant et nom, lus par le trait
`AnnuaireComptes`), établissement, cible, contexte, et l'**horodatage d'autorité**. Le
`horodatage_client`, s'il existe, est rendu à part et jamais présenté comme la date de l'action.

**Aucun point d'entrée d'écriture** — research R-17. Une entrée s'écrit dans la transaction de
l'opération qu'elle trace, par le trait `JournalAudit`.

**Ni export ni alertes** : DIR-04, tranche T5 (FR-040).

---

## Codes d'erreur métier introduits

| Code | Statut | Où | Ce qu'il dit |
|---|---|---|---|
| `identifiants_invalides` | `401` | 1, 2 | **Le seul code d'échec d'authentification.** Ne distingue jamais compte inconnu, mot de passe faux, compte désactivé ni dépassement de tentatives |
| `session_invalide` | `401` | 2 | Jeton de rafraîchissement inconnu, révoqué ou déjà consommé |
| `permission_absente` | `403` | 7→19 | L'appelant n'a pas la permission. **L'interface ne devrait jamais le provoquer** : l'action est absente sans permission (FR-026) |
| `methode_non_implementee` | `422` | 1 | `OTP_SMS` — refus **nommé** (FR-008) |
| `identifiant_absent` | `422` | 10 | Ni téléphone ni email |
| `identifiant_refuse` | `422` | 10, 12 | La création est impossible — **sans dire pourquoi** |
| `portee_incompatible` | `422` | 15 | `etablissement_id` fourni pour `admin_editeur`, ou absent pour un rôle d'établissement |
| `etablissement_inconnu` | `404` | 15 | Vérifié par trait, jamais par clé étrangère |
| `derniere_habilitation` | `409` | 16 | Le retrait laisserait l'établissement sans habilitation (FR-023) |

**Chaque code a sa clé i18n en `fr` et en `en`**, et chaque phrase passe par
`docs/design/lexique.md` **avant** d'être codée (module doré, septième couche, point 3). Le front
branche sur le `code`, **jamais** sur le `message` — qui est du diagnostic anglais nommant des
tables, et n'est jamais affiché.

---

## Sécurité déclarée au contrat

Les dix-sept opérations authentifiées portent `security(("bearer" = []))`. Les deux opérations
publiques — `session_ouvrir` et `session_rafraichir` — ne le portent pas, et **c'est la seule
liste d'exceptions du produit** : le test d'isolation P-08 la connaît nommément et échoue si une
opération nouvelle s'y ajoute sans décision.
