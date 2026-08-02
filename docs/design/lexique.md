# Kaya — Lexique du vocabulaire utilisateur

*Source de vérité du vocabulaire visible par l'utilisateur. Extrait de `docs/Kaya_Design.md` §6
le 2026-07-30 — ce fichier fait foi, `Kaya_Design.md` y renvoie.*

**Version 1.3.0** — deux entrées pour le geste qui manquait au produit : **quitter son poste**.
`fermerSession()` existait depuis le cycle CPT sans aucun appelant — il n'y avait, littéralement,
aucun moyen de sortir de sa session. Le mot retenu n'est pas « se déconnecter » : sur un terminal
partagé, l'appareil ne bouge pas, c'est la personne qui change, et le journal d'audit du §8.3 —
« ce que le propriétaire achète » — devient faux dès que Yao travaille sous le nom d'Aminata. Le
libellé nomme donc le geste réel, **passer la main**, et la seconde entrée porte son unique refus.

**Version 1.2.0** — le vocabulaire du cycle CPT : compte, personne, appareil connecté, registre des
actions, et **la phrase unique des deux échecs d'authentification**. Quatre mots y sont écartés
nommément — « rôle », « permission », « jeton » et « JWT » —, et l'entrée la plus contraignante du
lexique y apparaît : une phrase qui doit rester **la même** dans deux situations différentes, sans
quoi l'interface publie la liste des comptes existants.

**Version 1.1.0** — cinq entrées ajoutées avec la couche d'écriture d'ETB-02 : l'ajout et le retrait
d'un service, les deux refus qu'ils produisent, et la règle « le `message` de diagnostic n'apparaît
jamais ». Deux d'entre elles écartent un mot faux plutôt qu'un mot technique — « désactiver » décrit
un interrupteur, « supprimer » serait **faux**.

---

Le produit manipule des concepts fiscaux et techniques réels. L'utilisateur ne doit jamais les rencontrer sous leur nom d'origine.

| Concept interne | Ce qu'affiche l'interface |
|---|---|
| Certification FNE | « Envoi aux impôts » / « Validée par les impôts » |
| Document en état `SOUMISE` | « Envoi en cours… » |
| Document en état `INDETERMINEE` | « Nous ne savons pas si les impôts ont reçu cette facture » |
| Document en état `ECHEC` | « Les impôts ont refusé cette facture » + motif en clair |
| Stickers FNE restants | « Factures restantes » avec le nombre |
| Idempotence, rejeu, file d'attente | **N'apparaît jamais.** L'utilisateur voit « en attente d'envoi » et un nombre |
| Écriture orpheline, réconciliation | « Une consommation est arrivée après la facture » |
| Classe hors-ligne A/B/C/D | **N'apparaît jamais.** L'utilisateur voit « disponible hors connexion » ou « nécessite internet » |
| Taxe communale de nuitée | « Taxe de séjour (mairie) » — le nom légal reste sur la facture |
| Rebascule de palier de passage | « Durée dépassée : passé au tarif 4 h » |
| Temps de remise en état | « Chambre indisponible 30 min (ménage) » |
| Tenant, établissement | « Votre établissement » — le mot « tenant » n'existe pas pour l'utilisateur |
| Module d'activité | « Vos services » |
| Activation d'un module (`PUT … actif: true`) | « **Ajouter un service** » / *Add a service* — jamais « activer », qui décrit un interrupteur technique là où l'exploitant ajoute quelque chose à ce qu'il propose |
| Désactivation d'un module (`PUT … actif: false`) | « **Retirer** » / *Remove* — jamais « désactiver » (même motif), et **jamais « supprimer »**, qui serait faux : la désactivation ne supprime rien, et la réactivation restitue tout. La phrase de confirmation le dit : « Rien n'a été supprimé : vous pourrez le remettre » |
| `desactivation_bloquee` | « **Ce service est encore en cours d'utilisation.** » + ce qui l'occupe, compté. Jamais « obstacle », qui est le mot du trait `ObstacleDesactivation` |
| `module_non_implemente` | « **Ce service n'est pas encore disponible.** » — le référentiel le connaît, le produit ne le sert pas encore. À distinguer de « ce service n'existe pas » (`module_inconnu`) |
| Code d'erreur HTTP, `message` de diagnostic | **N'apparaît jamais.** L'interface branche sa clé i18n sur le `code`, jamais sur le `message` — qui nomme des tables et parle anglais technique |
| Unité louable | « Chambre » en hôtel, « logement » en résidence, « salle » pour la réunion — selon le contexte |
| RBAC, permissions | « Ce que chacun peut faire » |
| Synchronisation | « Enregistré » / « En attente d'envoi (4) » / « Hors connexion » |
| Attestation d'intégrité, enrôlement | « Téléphones autorisés » |
| `note_etablissement` | « **Note interne** » / *Internal note* — jamais « note d'établissement » : le §6 pose déjà que l'utilisateur est toujours dans le sien, le mot serait superflu sur un bouton |
| `capacite` | **N'apparaît jamais.** Le mot est un terme d'architecture — il nomme le transverse (stock, livraison, fidélité) par opposition au module d'activité. L'utilisateur ne voit que la **capacité concrète**, sous le service qui la consomme |
| `STOCK` (capacité) | « **Suivi du stock** » / *Stock tracking* — affiché sous le service qui le consomme, jamais comme une rubrique à part |
| `point_de_vente` | « **Point de vente** » / *Point of sale* |
| `point_de_vente` sans `table_pdv` | « **Comptoir** » / *Counter* — l'absence de tables **est** le comptoir. Jamais « point de vente sans tables », qui décrit un manque là où il s'agit d'une forme normale |
| Valeur héritée d'un niveau supérieur | « **Vaut pour tous vos établissements** » / *Applies to all your establishments* — jamais « hérité », « valeur par défaut » ni « niveau tenant » |
| Valeur surchargée au niveau courant | « **Modifié ici** » / *Changed here* — jamais « surcharge », « override » ni « exception » |
| `compte` | « **Compte** » / *Account* — ce avec quoi on se connecte. Distinct de la personne : une femme de ménage a une fiche et pas de compte, un comptable externe a un compte et pas de contrat |
| `personne` | « **Personne** » / *Person* — l'identité civile. Jamais « utilisateur », qui suppose un compte, ni « employé », qui suppose un contrat |
| `employe` | « **Employé** » / *Employee* — **n'apparaît nulle part au MVP**. La table est une provision (CPT-05) sans écran ; l'entrée est ici pour que le mot ne soit pas employé à la place de « personne » |
| `role`, `compte_role`, `permission` | « **Ce que chacun peut faire** » — règle déjà posée pour le RBAC. **Les mots « rôle » et « permission » n'atteignent jamais l'interface** : on montre ce qui est possible, pas la mécanique qui l'autorise |
| `journal_audit` | « **Registre des actions** » / *Activity log* — jamais « journal d'audit », qui est le nom technique et sonne comme une inspection. C'est ce que le propriétaire consulte pour savoir qui a fait quoi |
| Session, jeton d'accès, jeton de rafraîchissement, JWT | **N'apparaît jamais.** L'utilisateur voit un « **appareil connecté** » ; les quatre mots sont de la mécanique interne |
| Une session de la liste | « **Appareil connecté** » / *Connected device* — avec l'appareil, la première connexion et la dernière activité. Jamais « session » |
| Révocation d'une session | « **Déconnecter cet appareil** » / *Disconnect this device* — jamais « révoquer », qui est le mot du jeton. La phrase de confirmation dit l'effet : « Cet appareil devra se reconnecter » |
| Fermeture de **sa propre** session sur le terminal qu'on a sous la main (`DELETE /api/v1/session`) | « **Passer la main** » / *Hand over* — jamais « Se déconnecter », qui décrit la rupture d'un lien technique là où le geste réel est **de rendre le poste au suivant**. Au comptoir de Deloria, l'appareil ne bouge pas : c'est la personne qui change. L'infobulle dit l'effet : « **La personne suivante devra entrer son identifiant.** » / *The next person will have to enter their ID.* — jamais « votre session sera fermée », ni « vous serez déconnecté ». À ne pas confondre avec « Déconnecter cet appareil » ci-dessus : celui-là coupe un **autre** appareil, à distance, depuis la liste ; celui-ci rend **celui-là même** qu'on tient |
| Refus de passer la main, file d'envoi non vide | « **Des enregistrements ne sont pas encore partis.** Attendez le retour du réseau avant de passer la main. » / *Some entries haven’t been sent yet. Wait for the network before handing over.* — le mot « file » n'apparaît pas (règle déjà posée pour l'idempotence et le rejeu), et le refus est **immédiat**, jamais un échec après coup |
| `identifiants_invalides` (401) | « **Identifiant ou mot de passe incorrect** » / *Incorrect ID or password* — **une seule phrase, employée dans les deux cas** : compte inconnu et mot de passe faux. Deux phrases distinctes publieraient la liste des comptes existants (FR-012). C'est aussi pourquoi le compte désactivé et le dépassement de tentatives rendent **cette même phrase** |
| `session_invalide` (401) | « **Votre session a expiré. Reconnectez-vous.** » / *Your session has expired. Please sign in again.* |
| `mot_de_passe_refuse` (422) | Deux phrases distinctes, parce que l'utilisateur doit savoir quoi corriger : « **Choisissez un mot de passe d'au moins 8 caractères.** » ou « **Ce mot de passe est trop courant. Choisissez-en un autre.** » Jamais « compromis » ni « figurant dans une fuite », qui alarment sans instruire |
| `identifiant_refuse` (422) | « **Cet identifiant ne peut pas être utilisé.** » / *This ID cannot be used.* — **ne dit pas qu'il existe déjà**, ce qui reviendrait à confirmer un compte |
| `identifiant_absent` (422) | « **Indiquez un numéro de téléphone ou une adresse e-mail.** » |
| `portee_incompatible` (422) | « **Choisissez l'établissement concerné.** » — ou, pour l'administrateur éditeur, « **Ce compte agit sur tous les établissements.** » Jamais « portée », qui est le mot de la colonne |
| `derniere_habilitation` (409) | « **Il doit rester au moins une personne pouvant gérer les accès de cet établissement.** » — jamais « dernière habilitation » |
| `permission_absente` (403) | **Ne devrait jamais s'afficher** : sans le droit, l'action est **absente** de l'écran (FR-026). La phrase existe pour l'appel direct : « **Cette action ne vous est pas accessible.** » |
| Refus hors ligne d'une opération de classe C | Réemploi exact de la formulation d'ETB-02 : « **Cette action nécessite internet.** » / *This action requires an internet connection.* — annoncée **avant** la saisie, jamais après un échec |
| `methode_non_implementee` (422) | « **Ce compte se connecte autrement.** » — `OTP_SMS` est au référentiel et n'est pas servi ; jamais « méthode non implémentée » |

**Deux termes fiscaux conservés tels quels — règle 2 ci-dessous.** `classement` (« non classé »,
« résidence meublée », le nombre d'étoiles) et **« numéro de compte contribuable (NCC) »** gardent
leur nom officiel à l'écran comme sur les documents. Ce ne sont pas des noms techniques traduits en
jargon : ce sont les termes que l'administration emploie, que l'exploitant lit sur ses propres
papiers, et qu'il reconnaîtrait mal sous une reformulation. Consigné ici explicitement pour qu'une
relecture future ne les prenne pas pour un oubli d'entrée au lexique.

**Règle** : tout nouveau concept technique visible par l'utilisateur entre **dans ce fichier**
avant d'être codé. Fait partie de la Definition of Done (`docs/user-stories-v1.md` §0.4)
et de la porte **P-16** de la constitution.

---

## Comment ajouter une entrée

1. Le terme apparaît dans un bouton, un message, un libellé, une notification ou un document
   **non fiscal** → il lui faut une entrée ici **avant** d'être codé.
2. Le vocabulaire fiscal officiel — « facture normalisée », « taxe communale de nuitée » —
   reste sur les **documents légaux** et nulle part ailleurs. Sur un bouton, il passe par ce
   lexique.
3. Écrire la formulation telle qu'Adjoua la dirait à Abengourou, pas telle que la documentation
   technique la nomme.
4. Les deux clés i18n (`fr` puis `en`) sont créées dans le même changement — jamais de chaîne
   en dur (porte **P-16**).
5. **Une phrase qui doit rester identique dans deux situations se déclare comme telle.** Le cycle
   CPT en apporte la première : `identifiants_invalides`. La tentation permanente sera de la
   préciser — « ce compte n'existe pas », « mot de passe incorrect », « compte désactivé » — et
   chacune de ces précisions rend la liste des comptes lisible par qui essaie des numéros. Une
   entrée qui porte la mention « **une seule phrase** » ne se scinde pas sans rouvrir FR-012.

## Voir aussi

- `docs/design/derivation.md` — de quel motif maquetté hérite chaque écran non maquetté
- `docs/Kaya_Design.md` §5 « Les neuf règles » — dont la règle 6, « zéro jargon »
- `docs/design/composants.md` — les 14 composants canoniques
