# Kaya — Lexique du vocabulaire utilisateur

*Source de vérité du vocabulaire visible par l'utilisateur. Extrait de `docs/Kaya_Design.md` §6
le 2026-07-30 — ce fichier fait foi, `Kaya_Design.md` y renvoie.*

**Version 1.5.1** — complément des cinq entrées ci-dessous, trouvé à l'analyse de cohérence du
cycle SYN, le 2026-08-02. Trois manques, chacun réel :

- **la dérive d'horloge n'avait qu'un sens.** La phrase disait « retarde de {n} minutes », alors que
  la détection porte sur la **valeur absolue** de l'écart (SYN-04) : une horloge **en avance** — le
  cas du scénario de recette — n'avait aucune formulation, donc la moitié des cas était muette ;
- **les formulations anglaises manquaient** aux cinq libellés nouveaux, alors que le reste du
  document en donne ;
- **le titre de `S1` ne disait rien de sa route.** « Synchronisation » est proscrit du visible, et
  une URL est visible : le mot serait rentré par la porte du nom de fichier.

**Version 1.5.0** — le vocabulaire du cycle SYN : les **quatre formulations** que le témoin et le
panneau d'envoi réclamaient — connexion faible, saisie refusée, dérive d'horloge, titre de `S1` —
et la **confirmation que le lexique prime sur `app/core/i18n`**, qui avait dérivé sur les trois
libellés du témoin. Ajoutée le 2026-08-02.

**Version 1.4.0** — le vocabulaire du cycle HEB : la **formule**, le **type de chambre**, les cinq
refus du moteur de disponibilité, et le choix fiscal que l'exploitant fait à l'écran. Le mot
« formule » était déjà sur la maquette `G2` — « Vos formules », « Ajouter une formule » — et absent
d'ici : il est inscrit avant d'être codé. Six mots sont écartés nommément — « unité louable »,
« catégorie d'unité », « occupation », « intervalle », « palier » et « exclusion » —, et l'entrée
la plus délicate du cycle est celle du choix fiscal : **ses deux formulations ne disent rien des
personnes**, ce qui est précisément ce qui les rend employables alors que l'axe « par client » de
la taxe de séjour n'est pas tranché (B-10 du cadrage, échéance avant le cycle SEJ).

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
| Synchronisation | « Enregistré » / *Saved* · « En attente d'envoi (4) » / *Pending send (4)* · « Hors connexion » / *No connection* — **ces trois libellés font foi ; `app/core/i18n` disait « Connecté », « Hors ligne », « {n} éléments en attente », ce qui décrit le RÉSEAU au lieu de dire ce qui compte pour Aminata : son travail est-il en sécurité** |
| Réseau **dégradé** (le mot n'apparaît jamais) | « **Connexion faible** » / *Weak connection* — le réseau répond mal, les envois partent lentement. « Dégradé » est un terme d'ingénieur ; « faible » est ce qu'on dit spontanément d'un réseau qui rame |
| Écriture **définitivement refusée** par le serveur | « **Cette saisie a été refusée** » / *This entry was refused* **suivi du motif en clair et de ce qui reste possible.** Même patron que l'échec de certification : on dit qui refuse et pourquoi, jamais « erreur » seul. Ne jamais employer « rejet », « échec de synchronisation » ni un code |
| **Dérive d'horloge** au-delà du seuil | **Deux formes, une par sens** — « **L'heure de cet appareil retarde de {n} minutes.** » / *This device's clock is {n} minutes behind.* ou « **L'heure de cet appareil avance de {n} minutes.** » / *This device's clock is {n} minutes ahead.*, suivies **dans les deux cas** de « **Les durées et les montants restent calculés sur l'heure du serveur.** » / *Durations and amounts are still calculated from the server's time.* — **la seconde phrase est obligatoire** : sans elle, l'exploitant croira ses passages mal facturés, alors que l'horodatage d'autorité les protège (principe IV). **Les deux sens sont dus** : la détection porte sur la **valeur absolue** de l'écart (SYN-04), et une horloge en avance est aussi fausse qu'une horloge en retard — une seule forme laisserait la moitié des cas sans phrase |
| Titre de l'écran `S1` — panneau de synchronisation | « **Mes envois** » / *My uploads* — jamais « Synchronisation », le mot est proscrit par la ligne ci-dessus. Court, possessif : c'est son travail qui est en jeu, pas un mécanisme. **La route de la page suit le titre** (`/mes-envois`) : une URL est visible dans la barre d'adresse, et le mot proscrit ne s'y invite pas par la porte du nom de fichier |
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
| `formule` | « **Formule** » / *Rate plan* — ce qu'on vend sur un type de chambre : la nuitée, le passage, la demi-journée, le mois. Le mot est sur la maquette `G2` (« Vos formules », « Ajouter une formule ») et manquait ici. Jamais « tarif », qui ne désigne que le prix, ni « produit » ni « offre », qui sont les mots du catalogue |
| `categorie` (d'unité louable) | « **Type de chambre** » / *Room type* — « type de logement » en résidence, « type de salle » pour la réunion, selon le même contexte que l'entrée « Unité louable ». **Jamais « catégorie d'unité »**, qui est le nom de la table : il colle deux mots techniques dont l'un — « unité » — est déjà écarté ci-dessus |
| `occupation`, `intervalle`, `palier`, contrainte d'`exclusion` | **N'apparaît jamais.** Ce sont les mots de la table, de la période, du barème et de la garantie de base. L'utilisateur voit « chambre prise », « du … au … », « à partir de 4 h » et « déjà prise sur cette période » |
| `unite_deja_occupee` (409) | « **Cette chambre est déjà prise sur cette période.** » / *This room is already taken for this period.* — jamais « conflit », « chevauchement » ni « violation de contrainte », qui nomment la mécanique. Le refus vient de la base ; ce que l'utilisateur en lit est un fait d'exploitation |
| `formule_hors_categorie` (422) | « **Cette formule ne s'applique pas à cette chambre.** » / *This rate plan does not apply to this room.* |
| `plage_non_fractionnable` (422) | « **Une demi-journée se loue en entier : 8 h – 12 h ou 13 h – 16 h.** » / *A half-day is booked in full: 8 a.m. – 12 p.m. or 1 p.m. – 4 p.m.* — les deux plages sont **celles de l'établissement**, jamais écrites en dur : la phrase les reçoit. Jamais « non fractionnable », qui est le mot du code |
| `intervalle_invalide` (422) | « **La fin doit être après le début.** » / *The end must be after the start.* |
| `duree_hors_contrainte` (422) | « **Cette formule se loue de 1 h à 8 h.** » / *This rate plan is booked from 1 to 8 hours.* — les deux bornes viennent de la formule, jamais d'une constante |
| `formule.assujettie_taxe_nuitee` | « **Taxe de séjour comprise dans le prix** » quand elle vaut vrai, « **Pas de taxe de séjour sur cette formule** » sinon — les deux mentions exactes de la maquette `G2`. Jamais « assujettie », qui est le mot du formulaire fiscal |
| `regle_conversion_taxe = une_nuitee_par_occupation` | « **Une seule taxe pour tout le séjour** » / *One tax for the whole stay* — **formulation validée au terrain le 2026-08-02**. Ni « conversion », ni « règle », ni le nom de l'énumération n'atteignent l'interface |
| `regle_conversion_taxe = au_prorata` | « **Une taxe par nuit** » / *One tax per night* — même validation. ⛔ **Ces deux formulations ne disent rien des personnes**, et c'est ce qui les rend employables aujourd'hui : la taxe est due « par nuitée **et par client** » (cadrage §9.6), l'axe des personnes n'est pas tranché (B-10), et une phrase qui l'évoquerait préjugerait de l'arbitrage. Elles tranchent l'axe des nuits, rien d'autre |

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
- `docs/design/composants.md` — les composants canoniques (seize au 2026-08-02)
