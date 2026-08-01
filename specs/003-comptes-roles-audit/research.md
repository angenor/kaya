# Phase 0 — Recherche : comptes, rôles cumulables et journal d'audit

**Cycle 003 (CPT)** · tranche T1 · 2026-08-01

*Dix-neuf décisions. Chacune porte ce qu'on a retenu, pourquoi, et ce qui a été écarté. Aucune ne
propose de numéro de version : le gel de `docs/versions-gelees.md` est repris tel quel — `argon2`
**0.5.3** et `jsonwebtoken` **11.0.0** y figurent déjà au §3.1, nommément « (CPT-01) ».*

---

## R-01 — Où vivent les sessions : Redis, et la révocation est IMMÉDIATE

> **Révisée le 2026-08-01.** La première rédaction faisait du jeton d'accès un jeton non
> révocable, vérifié à la seule signature, et acceptait que la révocation attende le
> rafraîchissement suivant. **CPT-01 tranche l'inverse**, et le cadrage §12.2 l'imposait déjà :
> « coupure immédiate au départ d'un employé ». Avec un jeton d'une heure, attendre l'expiration
> ne la donne pas. La version ci-dessous remplace la précédente.

**Décision — trois clés Redis, et une lecture par requête :**

| Clé | Contenu | Expiration |
|---|---|---|
| `session:{session_id}` | La session : compte, établissement actif, appareil, famille de jetons | **90 jours** (durée du rafraîchissement) |
| `revoquees:{session_id}` | Marque de révocation, consultée **à chaque requête authentifiée** | 60 min — la durée maximale de vie d'un jeton d'accès encore en circulation |
| `famille:{famille_id}` | Le dernier jeton de rafraîchissement émis, pour la détection de réutilisation | 90 jours |

**Les cinq paramètres sont au « Récapitulatif des paramètres d'établissement »** de
`docs/user-stories-v1.md`, et le principe I·c les y rendait obligatoires : indicatif par défaut,
méthode d'authentification, longueur minimale du mot de passe, **durée du jeton d'accès (60 min)**,
**durée du jeton de rafraîchissement (90 jours, avec rotation à chaque usage)**.
`backend/tests/parametres_catalogue.rs` fait échouer le build si une clé du catalogue manque au
récapitulatif.

**Pourquoi 60 minutes, et non 30.** Ce n'est *pas* la brièveté du jeton qui porte la sécurité de
révocation — c'est la liste Redis. Une fois cette liste posée, la durée n'a plus à être courte, et
la rallonger est un gain réel : chaque rafraîchissement est un aller-retour réseau qui peut
échouer, et le réseau d'Abengourou est intermittent. Espacer les rafraîchissements réduit le
nombre d'occasions de tomber.

**Pourquoi 90 jours de rafraîchissement, et ce que cela oblige.** La durée vient d'un persona
réel : **M. Diarra, comptable externe, « vient une fois par mois »** — à 30 jours, il se
reconnecterait à chaque visite. La contrepartie n'est pas négociable :

- **rotation à chaque usage** — un jeton consommé ne se réemploie jamais ;
- **détection de réutilisation** — un jeton consommé deux fois signifie qu'une copie circule.
  Alors on révoque **toute la famille**, pas seulement le jeton présenté. Révoquer le seul jeton
  laisserait le voleur *et* la victime en course, et le premier des deux gagnerait ;
- **la déconnexion à distance de CPT-01 est opérationnelle dès ce cycle**, pas reportée : avant
  l'enrôlement d'appareil de CPT-05 (tranche T4), c'est **le seul recours** contre un téléphone
  volé. À 90 jours sans elle, un téléphone perdu garde l'accès un trimestre.

**Ce que « reconstructible » veut encore dire.** Le registre §9 range sessions, JWT et refresh
dans « éphémère Redis reconstructible ». La lecture par requête ne change pas ce statut : si Redis
disparaît, aucune session n'est retrouvée, tout le monde se reconnecte, **et aucune donnée métier
n'est perdue**. Ce qui change est l'exigence de disponibilité — Redis passe de « utile » à « sur
le chemin de chaque requête ». C'est le prix de la coupure immédiate, et il est assumé
explicitement plutôt que découvert à la première panne.

**Écarté** — table `session` en Postgres : une écriture Postgres par rafraîchissement sur un
chemin chaud, une sauvegarde de données sans valeur métier, et une contradiction directe avec le
registre §9. **Écarté** — porter la révocation par la seule brièveté du jeton : il faudrait
descendre à quelques minutes pour approcher l'immédiateté, ce qui multiplierait par dix les
aller-retours sur le réseau le plus mauvais du produit.

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

### La politique de mot de passe, et ce qu'elle rend obligatoire

Le récapitulatif tranche : **8 caractères, aucune règle de composition, refus des mots de passe
compromis.**

**Imposer majuscule + chiffre + symbole est refusé**, et ce n'est pas un assouplissement : cela
produit un mot de passe écrit sur un post-it au comptoir de la réception, à la vue des clients.
La longueur fait le travail, Argon2 fait le reste.

**Mais à 8 caractères sans composition, le refus des mots de passe compromis n'est plus
optionnel — c'est lui qui fait tout le travail.** Sans lui, `12345678` passe la politique.
Deux conséquences d'implémentation, non négociables :

- **liste embarquée dans le binaire**, jamais un appel réseau. Vérifier un mot de passe ne peut
  pas dépendre d'un tiers joignable : le jour où il ne répond pas, soit on bloque toutes les
  créations de compte, soit on accepte tout — et c'est le second qui arrive ;
- la vérification porte sur la **création et le changement** de mot de passe, jamais sur la
  connexion. Refuser à la connexion un mot de passe devenu compromis enfermerait dehors un
  utilisateur légitime sans lui donner de recours.

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
rafraîchissement suivant, soit **au plus 60 minutes**. C'est acceptable pour un **retrait de
droit**, qui est une décision d'organisation ; ce ne l'est pas pour un départ d'employé ou un
téléphone volé — et c'est précisément ce que la **révocation de session couvre, elle,
immédiatement** (R-01). Les deux mécanismes ne se remplacent pas : le premier ajuste des droits,
le second coupe un accès.

> **Faute à ne pas commettre** : croire que révoquer les sessions à chaque changement de rôle
> réglerait le délai. Cela déconnecterait Adjoua de ses trois postes chaque fois qu'on ajuste une
> permission — et l'équipe apprendrait à ne plus toucher aux rôles.

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

## R-18 — La file hors ligne ne connaît pas le jeton, et le retour du réseau a un ordre

**Décision.** Les écritures de **classe A** partent en file locale **sans jeton**. Au retour du
réseau, la séquence est **rafraîchir d'abord, vider ensuite** — jamais l'inverse. Une seule
fonction porte cet ordre, et la file n'a aucun autre chemin de sortie.

**Pourquoi.** Le jeton d'accès dure 60 minutes ; Aminata prend des commandes pendant une coupure
de 90. Si la file exigeait un jeton valide au moment de la **mise en file**, aucune commande ne
partirait ; si elle tentait de vider avant de rafraîchir, chaque élément partirait avec un jeton
expiré et reviendrait en `401`.

**Ce qui rend ce défaut particulièrement coûteux** : il ne se voit pas en test. En développement,
la coupure dure trente secondes et le jeton est encore valide au retour — tout passe. Il se
manifeste à Abengourou, un soir de service, et il perd **un service entier**. C'est la raison pour
laquelle la règle est écrite dans CPT-01 plutôt que laissée au bon sens de l'implémentation.

**Trois conséquences de conception :**

1. **La file ne stocke aucun jeton.** Un jeton mis en file avec l'élément serait périmé au retour,
   et le stocker prolongerait la durée de vie d'un secret sur un terminal.
2. **L'échec du rafraîchissement ne vide pas la file** — elle reste intacte et l'utilisateur voit
   qu'il doit se reconnecter. Vider en `401` détruirait les écritures qu'on cherche à sauver.
3. **Cette règle ne vaut que pour la classe A.** Les classes B, C et D n'entrent jamais en file
   (principe VI) ; leur refus est immédiat et explicite, avant toute saisie.

**Test** : `app/tests/file-jeton-expire.spec.ts` — coupure simulée plus longue que la durée du
jeton, trois écritures A en file, retour du réseau, et vérification que **le rafraîchissement
précède le premier envoi**. Le test échoue si l'ordre s'inverse, y compris quand les deux
réussissent.

---

## R-19 — Le nommage réservé des clés monétaires en JSONB

**Décision.** Toute clé monétaire d'un document JSON du produit porte le suffixe **`_mineur`**,
une **valeur entière**, et exige une clé **`devise`** au même niveau d'objet.

```json
{ "ecart_mineur": -12500, "devise": "XOF", "motif": "…" }
```

Jamais `12500.5`, jamais `"12 500 F"`, jamais un montant sans sa devise.

**Pourquoi — et c'est le point que le plan initial n'avait pas tiré jusqu'au bout.** Constater que
`journal_audit.contexte` « accueillera des montants » n'est pas une porte à ajuster : c'est le
**principe V qui cesse de tenir à la frontière du JSONB**. Un document JSON accepte un flottant là
où le principe impose un entier d'unité mineure, et rien dans le type de la colonne ne s'y oppose.

**Et l'endroit où cela arrive est le pire possible.** Le registre concerné trace les **écarts de
caisse**, les **modifications de tarif** et les **remises** — les trois choses que le propriétaire
consulte pour détecter une fraude. Un écart stocké en flottant, et l'audit ment sur le montant
qu'il est censé prouver. La constitution **1.6.0** étend P-10 en conséquence.

**Deux niveaux de vérification, parce qu'un seul ne suffirait pas :**

- **statique** — `scripts/ci/types-monetaires.sh` cherche les clés `*_mineur` dans le code et
  échoue sur toute affectation non entière, et sur tout montant JSON nommé autrement (`montant`,
  `prix`, `total` nus) ;
- **à l'écriture** — le service d'audit **valide le document** avant insertion : toute clé
  `*_mineur` doit porter un entier et être accompagnée de `devise`. Le contrôle statique ne voit
  pas ce qu'un service construit dynamiquement.

**Le versant positif**, sans lequel la porte passerait au vert en n'ayant rien à inspecter : un
cas de test écrit une entrée d'audit portant un montant, et vérifie qu'elle est **acceptée**
sous la forme entière et **refusée** en flottant comme en chaîne formatée.

---

## Ce que la phase 0 laisse ouvert

| Point | Pourquoi il reste ouvert | Quand il se ferme |
|---|---|---|
| **O-01** — `personne` en classe C et le check-in d'un client inconnu hors ligne | Le registre §12 le date « avant SEJ-02 », deux cycles plus loin. Ce cycle livre `personne` en **C** sans préempter | Cycle SEJ |

**Deux points ouverts ont été fermés le 2026-08-01, et ne le sont plus :**

| Point | Comment il s'est fermé |
|---|---|
| Durée des jetons | **Tranchée au récapitulatif** : accès **60 min**, rafraîchissement **90 jours avec rotation**. Ce ne sont plus des valeurs à choisir à l'implémentation, ce sont des paramètres d'établissement documentés, avec leurs contreparties obligatoires (R-01) |
| Extension du texte de P-05b | **Faite** — constitution **1.6.0**. P-05b porte désormais sur la **catégorie « registre immuable »** et non sur une liste de tables : l'outbox et le journal d'audit sont couverts, et le prochain registre le sera sans nouvel amendement. **P-10 a été étendue dans le même mouvement** (R-19) |
