# Phase 0 — Recherche : comptes, rôles cumulables et journal d'audit

**Cycle 003 (CPT)** · tranche T1 · 2026-08-01

*Dix-sept décisions. Chacune porte ce qu'on a retenu, pourquoi, et ce qui a été écarté. Aucune ne
propose de numéro de version : le gel de `docs/versions-gelees.md` est repris tel quel — `argon2`
**0.5.3** et `jsonwebtoken` **11.0.0** y figurent déjà au §3.1, nommément « (CPT-01) ».*

---

## R-01 — Où vivent les sessions : Redis, et rien d'autre

**Décision.** Le jeton de rafraîchissement vit en **Redis**, sous une clé par session
(`session:{compte_id}:{session_id}`), avec expiration native. Le jeton d'accès est **court et non
révocable individuellement** : il n'est vérifié qu'à la signature, sans aller-retour Redis.

**Pourquoi.** `docs/registre-classes-offline.md` §9 range « Sessions, JWT, refresh » dans « ce qui
n'est pas classé — **éphémère Redis reconstructible** ». Ce n'est pas un rangement administratif :
il dit que la perte de Redis reconnecte tout le monde sans perdre une donnée métier, donc que rien
n'y est sauvegardé et que rien n'en dépend pour la comptabilité. Le prompt de cadrage le répète —
Redis, « éphémère reconstructible SEULEMENT ».

**Conséquence assumée, écrite ici pour qu'elle ne soit pas redécouverte** : une révocation ne
coupe pas l'accès en cours, elle refuse le **rafraîchissement suivant**. C'est exactement ce que
FR-011 et l'hypothèse 5 de la spec énoncent. La durée du jeton d'accès est donc la borne
supérieure du délai de coupure, et elle devient un **paramètre d'établissement** (DoD point 9),
pas une constante.

**Écarté** — table `session` en Postgres. Elle donnerait une révocation instantanée et une liste
d'appareils durable, au prix d'une écriture Postgres par rafraîchissement sur le chemin le plus
chaud du produit, d'une sauvegarde de données sans valeur métier, et d'une contradiction directe
avec le registre §9. **Écarté** — vérification Redis à chaque requête : elle rendrait Redis
indispensable à toute lecture, ce qui n'est plus « reconstructible » mais « critique ».

---

## R-02 — Le point que l'on écrirait mal : l'indiscernabilité coûte un hachage factice

**Décision.** Quand l'identifiant présenté n'existe pas, le service **exécute quand même une
vérification Argon2** contre un condensat de référence fixe, puis rend le même refus que pour un
mot de passe faux.

**Pourquoi.** FR-012 exige que les deux échecs soient indiscernables « en message, en code de
retour **et en ordre de grandeur de temps de réponse** ». Un service qui rend `401` en 2 ms pour
un compte inconnu et en 90 ms pour un mot de passe faux publie la liste de ses comptes à qui sait
chronométrer — le message identique n'y change rien. C'est la faute classique, et elle ne se voit
sur aucune relecture de code.

Le condensat de référence est calculé **au démarrage** avec les mêmes paramètres que les
condensats réels ; le recalculer à chaque requête ne changerait rien au coût mais ajouterait une
source de variance.

**Test.** `backend/tests/authentification_indiscernable.rs` : 100 tentatives de chaque type,
comparaison des médianes, échec si le rapport sort d'un facteur 2. Un seuil en valeur absolue
serait inutilisable — la CI partagée n'a pas de temps stable.

---

## R-03 — Argon2id, paramètres explicites, jamais « par défaut »

**Décision.** Argon2**id**, `m = 19456` KiB, `t = 2`, `p = 1`, sel de 16 octets, sortie de
32 octets. Les paramètres sont **écrits dans le code** avec leur source, et le condensat les
porte (format PHC), ce qui rend une future montée possible sans invalider les mots de passe
existants.

**Pourquoi.** Ce sont les paramètres de la recommandation OWASP pour Argon2id, retenus parce
qu'ils tiennent sur le VPS Contabo cible sans transformer la connexion en attente perceptible.
Écrire « paramètres par défaut de la crate » serait une décision invisible : elle changerait à la
première montée de version, silencieusement, dans un sens ou dans l'autre.

**Rehachage à la connexion** : si le condensat lu porte des paramètres différents des paramètres
courants, le mot de passe est **rehaché après vérification réussie**. Sans cela, une montée de
paramètres ne protégerait que les comptes créés après elle.

---

## R-04 — La dérogation `CONTEXTE_PAR_EN_TETES` est levée par ce cycle, et c'est un coût réel

**Décision.** `backend/api/src/contexte.rs` cesse de lire `x-kaya-tenant` et `x-kaya-compte`.
`ContexteAppel` est extrait du **jeton d'accès vérifié**, et porte désormais `tenant_id`,
`compte_id`, `etablissement_id` actif et les **permissions effectives**. `verifier_derogation()`
et la variable `KAYA_CONTEXTE_PAR_EN_TETES` disparaissent.

**Pourquoi.** La dérogation est nommée, datée et porte sa condition de levée : « **CPT-01 — le
contexte vient du jeton vérifié, ces en-têtes disparaissent** ». C'est ce cycle. La laisser
ouverte serait la transformer en état permanent, ce que la constitution interdit explicitement.

**Le coût, chiffré pour qu'il ne surprenne pas `/speckit-tasks`** : les **21 opérations** du
contrat sont aujourd'hui testées par des requêtes qui posent deux en-têtes. Tous ces tests —
`isolation_tenant.rs` en tête, qui est la porte P-08 — doivent obtenir un **vrai jeton**. La
fonction d'aide vit dans `backend/tests/commun` et appelle le **vrai chemin de connexion** contre
un compte de seed ; forger un jeton directement avec la clé de test ferait passer les tests sans
jamais exercer l'authentification.

**Écarté** — garder les en-têtes « en mode test seulement ». Un chemin d'authentification
alternatif activable par variable d'environnement est exactement la porte dérobée que la
dérogation décrivait ; et il rendrait P-08 aveugle au chemin réellement servi.

---

## R-05 — La clé de signature vient de l'environnement, et le binaire refuse de démarrer sans

**Décision.** Une clé de signature symétrique **par déploiement**, lue dans l'environnement au
démarrage, jamais compilée. Absence ou longueur insuffisante → refus de démarrer, sur le modèle
exact de `verifier_derogation()` qu'elle remplace.

**Pourquoi.** Principe IX : « aucun secret dans le binaire Tauri » — et par extension aucun secret
dans une image. Le refus au démarrage plutôt qu'un repli sur une valeur par défaut : une clé de
développement laissée en production est le défaut le plus banal du domaine, et il ne se voit pas.

**Pas de clé par tenant.** Le coffre chiffré par tenant du cadrage §12.1 vise les **clés FNE et
les secrets d'agrégateur** — des secrets appartenant au client. La clé de signature appartient au
déploiement. Une clé par tenant imposerait de résoudre le tenant *avant* de vérifier le jeton,
c'est-à-dire de faire confiance à une partie non vérifiée du jeton pour choisir la clé qui le
vérifie.

---

## R-06 — Les permissions effectives voyagent dans le jeton, pas dans une requête par appel

**Décision.** Le jeton d'accès porte `compte_id`, `tenant_id`, `etablissement_id` actif et la
**liste des permissions effectives** pour cet établissement. Elles sont recalculées à chaque
délivrance — connexion et rafraîchissement.

**Pourquoi.** L'alternative — relire `compte_role` et `role_permission` à chaque requête — ajoute
deux lectures à toute opération du produit, y compris aux plus chaudes. Et le front a besoin de la
même liste pour filtrer les tuiles de `R1` : la calculer deux fois par deux chemins différents
serait la garantie qu'ils divergent.

**Conséquence, cohérente avec l'hypothèse 5 de la spec** : un rôle retiré ne prend effet qu'au
rafraîchissement suivant. C'est acceptable pour un **retrait**, pas pour une urgence — d'où la
révocation de session explicite (FR-011), qui, elle, est immédiate au rafraîchissement.

**Le front ne décode jamais le jeton.** La réponse de connexion porte les permissions en clair, à
côté des jetons. Décoder un JWT côté client pour y lire des droits, c'est apprendre à l'interface
à faire confiance à une charge utile qu'elle ne vérifie pas.

**Taille.** Les codes de permission sont courts (`etb.service.basculer`) et le MVP en compte moins
de trente. Si l'ensemble dépassait un jeton raisonnable, le repli documenté est de porter les
**rôles** et de résoudre les permissions côté serveur depuis un cache du référentiel — décision à
rouvrir alors, pas maintenant.

---

## R-07 — Un rôle est attribué par établissement, et le jeton en désigne un seul

**Décision.** `compte_role {compte_id, role_code, etablissement_id}`. La connexion accepte
optionnellement un `etablissement_id` ; sans lui, le premier établissement accessible par ordre
stable devient l'établissement actif. La réponse porte **la liste des établissements
accessibles**.

**Pourquoi.** M. Koffi possède deux établissements dont les besoins diffèrent, et un gérant de
l'un n'est pas gérant de l'autre. Un rôle global au tenant obligerait à réintroduire la portée
plus tard, sur une table déjà peuplée.

**Frontière avec ETB-06.** Le **sélecteur de contexte permanent** — bascule en deux tapes, sans
reconnexion — est ETB-06, P1, hors périmètre. Ce cycle livre le strict nécessaire : choisir
l'établissement **à la délivrance d'un jeton**. Rien de plus, et surtout pas un demi-sélecteur
qu'ETB-06 devrait défaire.

**Exception `admin_editeur`.** Ce rôle est de portée éditeur, sans établissement.
`etablissement_id` est donc **nullable**, et la contrainte d'unicité le traite en conséquence.

---

## R-08 — Le journal d'audit n'est pas un consommateur de l'outbox

**Décision.** `journal_audit` est un agrégat **distinct** du grand livre d'événements. Il s'écrit
dans la même transaction que l'opération tracée quand celle-ci est transactionnelle, et de façon
autonome pour les actions de classe A. Aucun consommateur outbox ne l'alimente.

**Pourquoi.** L'encadré du registre §5.2 est décisif : « **`journal_audit` est A, l'opération
qu'il trace garde sa propre classe.** Tracer une remise hors ligne est A ; appliquer la remise est
B. **Les deux ne voyagent pas ensemble.** » Dériver l'audit de l'outbox rendrait impossible de
tracer une **ouverture de tiroir** — explicitement de classe A au cadrage §11.3, donc effectuée
et tracée hors ligne — puisque l'outbox suit la transaction d'une opération qui, elle, n'a pas eu
lieu en ligne.

Deux registres, deux publics, écrit ici une fois pour toutes :

| | Grand livre d'événements (outbox) | Journal d'audit |
|---|---|---|
| Pour qui | Les consommateurs internes — métriques, notifications, reconstitution | **Le propriétaire**, à l'écran |
| Ce qu'il trace | **Toute** transition d'état métier | **Dix familles** d'actions sensibles |
| Classe | Suit l'opération | **A**, toujours |
| Écrit hors ligne | Non | **Oui** |

**Une action tracée produit donc les deux** : son événement outbox (P-05) *et* son entrée d'audit.
Ce n'est pas une duplication — c'est la conséquence directe de deux classes différentes.

---

## R-09 — La taxonomie d'audit est une énumération contractuelle, pas une table

**Décision.** Les dix familles vivent dans une **énumération Rust** exposée en `ToSchema`, donc
présentes au contrat OpenAPI et dans le client TypeScript généré. Elles sont documentées dans
`docs/taxonomie-audit.md`, et un test compare les deux dans le sens **code → document**.

**Pourquoi.** Le registre §9 range la taxonomie d'événements dans « versionné dans le dépôt, pas
en table ». Une table de types demanderait une migration pour chaque type nouveau et rendrait le
filtre du front dépendant d'une lecture ; une énumération contractuelle fait échouer la
**compilation du front** quand un type change de nom, ce qu'une table ne ferait jamais.

**Les huit types dus figurent dès maintenant dans l'énumération.** Ce n'est pas contraire au
principe X : CPT-04 les nomme tous les dix, ils sont donc du périmètre. Ce que le principe X
interdit, c'est d'écrire la logique qui les émet — et aucune ne l'est.

**Harnais à étapes dues** (patron du cycle 002) : `docs/taxonomie-audit.md` porte, pour chaque
type, l'état `branché` ou `dû par <story>`. `backend/tests/audit_taxonomie.rs` échoue si un type
est branché sans test, si un type dû acquiert un chemin d'écriture sans changer d'état, ou si le
document et l'énumération divergent.

---

## R-10 — L'immuabilité se tient par les privilèges, pas par une convention

**Décision.** `GRANT SELECT, INSERT ON comptes.journal_audit TO kaya_app` — **ni `UPDATE`, ni
`DELETE`**, exactement comme `note_etablissement` au module doré. Doublé d'un contrôle statique
jumeau de celui de l'outbox.

**Pourquoi.** Le patron du module doré le dit : « Accorder `UPDATE` casserait la commutativité que
le test de désordre vérifie — et le classement en A deviendrait faux sans que rien ne le
signale. » Pour l'audit, l'enjeu est plus direct encore : un journal qu'on peut réécrire n'a
aucune valeur pour celui qui l'achète.

**Extension de `scripts/ci/outbox-sans-purge.sh`.** La porte **P-05b** vise nommément le journal
d'événements ; son script gagne un **second contrôle**, déclaré dans son en-tête de périmètre,
qui applique la même recherche à `journal_audit`. Deux corollaires du § « Couverture des portes » :
le contrôle **déclare ce qu'il lit**, et il porte son **versant positif** — vérifier qu'aucune
suppression n'existe ne vaut rien si aucune entrée ne s'écrit.

> **Point porté à la revue de constitution, sans être tranché ici** : la formulation de P-05b ne
> mentionne que l'outbox. L'étendre au journal d'audit dans le texte relève de
> `/speckit-constitution`, pas de ce plan. Le contrôle est livré ; le texte de la porte est à
> amender séparément.

---

## R-11 — Le schéma `comptes` se crée par une migration nouvelle, jamais en modifiant `0001`

**Décision.** Migration `0014` : `CREATE SCHEMA comptes` + `GRANT USAGE ... TO kaya_app`, sur le
modèle des trois schémas de `0001` (`etablissements`, `synchronisation`, `fiscalite`).

**Pourquoi.** `0001` est appliquée ; la porte **P-02** compare l'empreinte de chaque migration
appliquée à celle du dépôt et fait échouer le build sur toute modification. Ajouter le schéma
« là où sont les autres » est le réflexe naturel et le plus coûteux.

---

## R-12 — Les tables de ce cycle sont dans `comptes`, et aucune clé étrangère n'en sort

**Décision.** `personne`, `compte`, `employe`, `compte_role`, `journal_audit`, `appareil_enrole`
dans le schéma `comptes` ; `role`, `permission`, `role_permission` également, en **référentiels
globaux** sur le régime nommé de `0008`. Aucune `REFERENCES` vers `etablissements.*`.

**Pourquoi.** Principe II et porte P-04. `compte_role.etablissement_id` **ne référence pas**
`etablissements.etablissement` — comme `note_etablissement.auteur_compte_id` ne référence pas
`comptes.compte`. Le module doré l'écrit comme « le point le plus contre-intuitif du patron » :
l'intégrité référentielle inter-modules passe par un **trait exposé**, jamais par la base.

L'existence de l'établissement est donc vérifiée **par le service**, via
`EstablishmentDirectory` — déjà exposé par `socle/etablissements` — avant l'attribution d'un rôle.
C'est ce qui donne un `404` au lieu d'une violation de contrainte, et c'est le sens du point 3 de
l'ordre des opérations du module doré.

**Le régime des trois référentiels** suit `0008` à la lettre : pas de `tenant_id`, ordre
impératif `CREATE TABLE` → `INSERT` → `ENABLE`/`FORCE` → `CREATE POLICY`, politique
`lecture_universelle` en `FOR SELECT USING (true)`, politique `administration_editeur` pour
`kaya_owner`, et `GRANT SELECT` seul à `kaya_app`.

---

## R-13 — La limitation des tentatives est en Redis, par identifiant **et** par origine

**Décision.** Compteur Redis à fenêtre glissante, sur deux clés distinctes : l'identifiant
présenté et l'adresse d'origine. Dépassement → même refus que toute autre tentative, avec un
délai. Aucun verrouillage définitif de compte.

**Pourquoi.** Compter seulement par identifiant laisse un balayage de mille comptes à une
tentative chacun ; compter seulement par origine laisse passer une attaque distribuée sur un
compte unique. Et **le refus doit rester indiscernable** (R-02) : un message « trop de
tentatives » sur un identifiant existant, et un `401` ordinaire sur un identifiant inconnu,
rétabliraient exactement la fuite que R-02 ferme.

**Pas de verrouillage définitif** : c'est un déni de service offert à quiconque connaît le
téléphone d'Adjoua. Le compteur expire.

---

## R-14 — Les seeds portent des comptes, et le mot de passe de démonstration refuse la production

**Décision.** Les seeds créent M. Koffi (`proprietaire`), Adjoua (`gerant` + `caissier` +
`receptionniste` — **les trois, c'est le point du cycle**) et Yao (`receptionniste`), avec un mot
de passe de développement lu dans l'environnement. Le binaire de seeds **refuse de s'exécuter**
si l'environnement se déclare production.

**Pourquoi.** TRX-05a impose des seeds idempotents et rejouables ; les comptes en font partie
depuis TRX-05b (« les comptes à CPT »). Un mot de passe de démonstration en dur dans un dépôt
finit toujours par tourner quelque part — le refus explicite est moins cher qu'une rotation.

**Idempotence.** Comme partout : identifiants UUID v7 **figés** dans les seeds et
`ON CONFLICT (id) DO NOTHING`. Des identifiants tirés au hasard rendraient les seeds non
rejouables, ce que la mécanique de TRX-05a interdit.

---

## R-15 — Dix types d'événements outbox, et la connexion n'en est pas un

**Décision.** Les transitions d'état de ce cycle émettent : `personne.creee`,
`personne.modifiee`, `compte.cree`, `compte.modifie`, `compte.desactive`, `compte.reactive`,
`compte.mot_de_passe_change`, `role.attribue`, `role.retire`, `session.revoquee`. Soit **dix**.
`employe.*` : **aucun**, la table est vide.

**Ce qui n'émet pas, et pourquoi** : la connexion, le rafraîchissement et l'échec
d'authentification. Ce ne sont pas des transitions d'état **métier** : leur état durable est nul,
leur trace utile est le journal applicatif, et les inscrire au grand livre transformerait un
registre comptable permanent en journal de trafic — avec, en prime, la liste horodatée des
présences du personnel dans un registre à rétention illimitée.

`session.revoquee` **est** émis : c'est un acte d'administration, décidé par quelqu'un contre
quelqu'un, et il a une valeur de reconstitution.

**Deux tenants, obligatoire.** L'exigence 5 du § « Couverture des portes » — née du défaut de
séquence corrigé par la migration `0012` — impose d'exercer **tout nouveau type d'événement sur
les deux tenants de démonstration**. Dix types nouveaux, donc dix × deux.

---

## R-16 — Ni `argon2` ni `jsonwebtoken` n'ajoutent de dépendance native — à vérifier, pas à supposer

**Décision.** Les deux crates sont déjà gelées au §3.1 et reprises telles quelles. La
**vérification d'architecture** est une tâche explicite du cycle : la première construction
`docker buildx build --platform linux/amd64` intervient **avant** que le front ne s'y appuie, pas
à la fin.

**Pourquoi.** Le poste de développement est `arm64`, la cible est `linux/amd64`, et « le binaire
Rust n'est pas multi-architecture ». Une dépendance qui ne se construit que sur l'une des deux se
découvre au premier `docker buildx`, et coûte d'autant plus cher qu'elle arrive tard. `argon2`
(RustCrypto) est du Rust pur ; la chaîne cryptographique de `jsonwebtoken` contient de l'assembleur
optimisé par architecture — **les deux cibles sont annoncées supportées, et c'est précisément le
genre d'affirmation qu'il faut constater plutôt que citer**.

**Aucune version n'est proposée ici.** Si l'une des deux devait bouger, c'est la revue mensuelle
du **2026-08-31** qui tranche.

---

## R-17 — L'entrée d'audit voyage avec l'opération qu'elle trace, donc aucun endpoint d'audit en écriture

**Décision.** Le journal d'audit n'expose **aucun point d'entrée d'écriture**. Une entrée s'écrit
dans la **transaction de l'opération tracée**, côté serveur, par le trait `JournalAudit`. Seule la
**lecture** est exposée (`GET /api/v1/journal-audit`).

**Pourquoi — et pourquoi ce n'est pas en contradiction avec la classe A.** L'encadré du registre
§5.2 décrit un cas précis : une opération B *sérialisée par un nœud de site* dont la trace remonte
au cloud séparément. **Le nœud de site est le mode C, incrément 3.** Au MVP, qui est en mode A
(cloud), aucune opération de classe B ni C ne s'exécute hors ligne : il n'existe donc **aucun cas
où une entrée d'audit voyage seule**. Pour une opération de classe A effectuée hors ligne —
l'ouverture de tiroir, due par IMP-01 — l'opération et sa trace remontent dans la **même
requête**, portées par le même UUID v7.

Livrer un endpoint d'écriture d'audit maintenant produirait exactement ce que le § « Couverture
des portes » dénonce : une cible vide qui passe toujours, et une surface par laquelle un terminal
pourrait forger des entrées dans le registre que le propriétaire achète.

**Conséquence sur les tests §0.7.** Le rejeu triple et le désordre (classe A) se vérifient **au
niveau du service d'audit**, contre la base réelle — le §0.7 exige « la même écriture envoyée
trois fois produit un seul enregistrement », il n'impose pas HTTP. La mise en file locale d'une
entrée d'audit isolée est **due par le mode C** et par le premier type traçable depuis un
terminal ; le harnais à étapes dues (R-09) porte cette échéance.

---

## Ce que la phase 0 laisse ouvert

| Point | Pourquoi il reste ouvert | Quand il se ferme |
|---|---|---|
| **O-01** — `personne` en classe C et le check-in d'un client inconnu hors ligne | Le registre §12 le date « avant SEJ-02 », deux cycles plus loin. Ce cycle livre `personne` en **C** sans préempter | Cycle SEJ |
| Durée exacte du jeton d'accès | Paramètre d'établissement (DoD 9), pas une constante. La valeur initiale se fixe à l'implémentation | `/speckit-tasks` |
| Extension du **texte** de P-05b au journal d'audit | La constitution s'amende par `/speckit-constitution`, jamais depuis un plan | Après ce cycle |
