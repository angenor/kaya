# Kaya — Registre des classes hors-ligne

*Source de vérité de la classe A/B/C/D de chaque entité et de chaque opération.
Référencé par le principe VI de `.specify/memory/constitution.md` et par le point 5 de la
Definition of Done (`docs/user-stories-v1.md` §0.4).*

**Version 1.0.0 — 2026-07-30**

---

## 1. Objet et autorité

Ce registre est **normatif**. Toute entité et toute opération qui écrit en base porte ici une
classe. La règle absolue du cadrage §11.1 s'applique sans exception :

> **Une opération B, C ou D atteignable depuis un chemin de code exécutable hors ligne FAIT
> ÉCHOUER LE BUILD.** Invariante vérifiée par test (SYN-01), pas par convention.

Ce fichier fait foi sur toute supposition de code. En cas de contradiction avec le classement
de référence de `docs/cadrage-v1.md` §11.3, **le cadrage prime** et ce registre est corrigé dans
le même changement.

**Une entité absente de ce registre est une entité non implémentable.** La déclarer ici fait
partie de la story qui l'introduit, pas d'un travail ultérieur.

## 2. Les quatre classes

| Classe | Critère | Autorité | Écriture hors ligne |
|---|---|---|---|
| **A** | Append-only, commutatif, sans contrainte d'unicité, sans effet monétaire | Aucune | **Oui** |
| **B** | Sérialisation requise, à l'échelle d'un établissement | Nœud de site (mode C) ou cloud | **Mode C seulement** |
| **C** | Référentiel partagé entre établissements, ou relation éditeur–client | Cloud | **Non** |
| **D** | Dépend d'un tiers (DGI, agrégateur de paiement) | Externe | **Non** |

## 3. Arbre de décision

S'arrêter à la première réponse « oui ». Les codes de branche (`D1`, `C2`, `B3`, `A4`) sont
employés comme justification dans tout le registre.

| Code | Question | Classe |
|---|---|---|
| **D1** | Dépend d'un tiers externe ? | **D** |
| **C2** | Modifie du référentiel partagé entre établissements, ou la relation éditeur–client ? | **C** |
| **B3** | Peut produire un conflit si deux utilisateurs du même établissement l'exécutent simultanément — ressource unique, numérotation, décrément, effet monétaire ? | **B** |
| **A4** | Sinon | **A** |

**En cas de doute, classer plus strictement.** Une entité indûment classée A produit des
incohérences silencieuses découvertes trois mois plus tard en pleine clôture ; une entité
indûment classée B produit une frustration immédiate, visible et corrigeable.

## 4. Comment déclarer une entité

1. Dérouler l'arbre du §3 et noter le **code de branche**, pas seulement la lettre.
2. Ajouter une ligne dans le tableau du crate propriétaire (§5 à §9).
3. Écrire les tests exigés par la classe (§11).
4. Si l'entité est une **provision** — table sans logique —, la déclarer au §10 et non au §5-9.
5. Consigner l'ajout au §13 (journal des modifications).

**Une même table peut porter deux classes selon l'opération.** C'est le cas normal, pas une
exception : `encaissement` est B en espèces et D en Mobile Money ; `ligne_commande` est A à la
saisie et B à l'annulation après envoi. Le registre classe **l'opération**, et la colonne
« Entité ou opération » le dit explicitement.

---

## 5. `socle/` — noyau agnostique

### 5.1 `socle/etablissements`

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `tenant` — création, modification | **C** | C2 — relation éditeur–client | ETB-01 |
| `etablissement` — création, modification | **C** | C2 — référentiel | ETB-01 |
| `etablissement.classement` (étoiles / non classé) | **C** | C2 — détermine le barème de nuitée | ETB-01, §11.3 |
| `etablissement.commune`, `.fuseau_horaire`, `.devise`, `.ncc` | **C** | C2 — référentiel fiscal | ETB-01 |
| `module_activite` — référentiel | **C** | C2 — référentiel partagé | ETB-02 |
| `etablissement_module` — activation, désactivation | **C** | C2 — modules activés | ETB-02, §11.3 |
| `capacite` — référentiel | **C** | C2 — référentiel partagé | ETB-02b |
| `profil_stock` — référentiel | **C** | C2 — référentiel partagé | ETB-02b |
| `module_capacite` — déclaration de consommation, `profil_stock` | **C** | C2 — référentiel | ETB-02b |
| `parametre_catalogue` — référentiel des clés de configuration | **C** | C2 — référentiel partagé | ETB-04 |
| `point_de_vente` — création, modification | **C** | C2 — référentiel | ETB-03 |
| `table_pdv` — création, modification du référentiel de tables | **C** | C2 — référentiel | ETB-03 |
| `parametre_configuration` — toute valeur de la chaîne d'héritage | **C** | C2 — référentiel de paramètres | ETB-04 |
| `branding` — logo, couleurs, en-têtes de documents | **C** | C2 — référentiel | ETB-05 |
| `note_etablissement` — création | **A** | A4 — append-only, commutative, sans effet monétaire | TRX-01 |
| Sélection d'établissement actif (contexte local) | **A** | A4 — préférence locale, sans effet | ETB-06 |
| **Lecture en cache** de tout référentiel et de tout paramètre ci-dessus | **A** | A4 — lecture seule, avec **fraîcheur affichée** | ETB-02, ETB-04 |

> **L'écriture et la lecture d'un référentiel ne sont pas de la même classe, et il faut le dire.**
>
> Toutes les écritures ci-dessus sont en **C** : aucun référentiel ne se modifie hors ligne. Mais
> leur **lecture** doit rester possible sans connexion, avec la date de dernière synchronisation
> affichée — sinon le produit devient inutilisable dès la première coupure. Une serveuse qui ne
> peut pas lire la liste des services de son établissement ne peut rien faire du tout, alors même
> qu'elle n'a rien à y modifier.
>
> C'est la même dualité que `encaissement`, **B** en espèces et **D** en Mobile Money (§5.3) : le
> registre classe des **opérations**, pas des tables. Sans cette ligne, un cycle ultérieur
> trancherait dans un sens ou dans l'autre sans que la décision soit visible — et le sens le plus
> probable serait « tout est C, donc rien ne se lit hors ligne ».
>
> **Le mécanisme de cache et le témoin de fraîcheur ne sont pas livrés par ETB** : ils relèvent de
> SYN-01/02 et d'ETB-06. Ce qui est arrêté ici est la **classe**, pour que le cycle qui écrira le
> cache n'ait pas à la deviner.

### 5.2 `socle/comptes`

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `personne` — création, modification | **C** | C2 — identité partagée entre établissements | CPT-00 |
| `compte` — création, modification | **C** | C2 — identité d'authentification | CPT-01 |
| `compte_role` — **attribution ou retrait de rôle** | **C** | C2 — explicitement C au cadrage §11.3 | CPT-02 |
| `role`, `permission` — référentiels | **C** | C2 — référentiel | CPT-02 |
| Élévation de privilège | **C** | C2 — **aucune élévation hors ligne, jamais** | CPT-02 |
| `appareil_enrole` — enrôlement, révocation | **C** | C2 — explicitement C au cadrage §11.3 | CPT-05 |
| Attestation d'intégrité — vérification | **C** | C2 — vérifiée côté serveur | CPT-06 |
| Relevé de position (géorepérage souple) | **A** | A4 — signal d'audit, jamais bloquant | CPT-06 |
| `journal_audit` — écriture d'une entrée | **A** | A4 — append-only, immuable, sans effet propre | CPT-04 |

> **`journal_audit` est A, l'opération qu'il trace garde sa propre classe.** Tracer une remise
> hors ligne est A ; appliquer la remise est B. Les deux ne voyagent pas ensemble.

> **Point de vigilance — client inconnu en mode C.** `personne` est C, donc un check-in
> (classe B, autorisé hors ligne en mode C) portant un **client jamais vu** exige le cloud pour
> créer sa fiche. Voir §12, décision ouverte O-01.

### 5.3 `socle/caisse`

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `caisse` — création, rattachement | **C** | C2 — référentiel | ETB-03 |
| `shift` — **ouverture**, fond de caisse déclaré | **B** | B3 — un utilisateur, une caisse, une période | CAI-01, §11.3 |
| `shift` — passation, comptage contradictoire | **B** | B3 — effet monétaire | CAI-01 |
| `encaissement` — **espèces** | **B** | B3 — irréversible, effet monétaire | CAI-02, §11.3 |
| `encaissement` — **virement**, **à crédit** | **B** | B3 — effet monétaire, constaté sans tiers en ligne | CAI-02 |
| `encaissement` — **Mobile Money**, **carte** | **D** | D1 — agrégateur de paiement | CAI-02, §11.3 |
| Règlement fractionné multi-modes | **classe de chaque part** | — | CAI-02 |
| `sortie_de_caisse` — dépense, avance, prélèvement | **B** | B3 — effet monétaire | CAI-03, §11.3 |
| `comptage`, `ecart_de_caisse` | **B** | B3 — effet monétaire, tracé | CAI-04, §11.3 |
| `cloture_shift` | **B** | B3 — sérialisation par caisse | CAI-05 |
| `cloture_journaliere` | **B** | B3 — **atomique**, explicitement B au cadrage §11.3 | CAI-06 |
| Ouverture de tiroir-caisse (tracée) | **A** | A4 — explicitement A au cadrage §11.3 | IMP-01, §11.3 |

### 5.4 `socle/fiscalite`

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `parametrage_fiscal` — taux, barèmes de taxe | **C** | C2 — référentiel fiscal | FIS-03, §11.3 |
| `cle_fne` — saisie, rotation (coffre chiffré par tenant) | **C** | C2 — explicitement C au cadrage §11.3 | FIS-04 |
| **Calcul** de la taxe de nuitée, de la TVA, de la taxe touristique | **A** | A4 — déterministe et local | §11.3 cas particulier |
| **Inscription** d'une taxe sur un document fiscal | **D** | D1 — passe par la certification | §11.3 cas particulier |
| `document_fiscal` (facture FNE) — émission | **D** | D1 — numérotation attribuée par la DGI | FIS-02 |
| `avoir` — émission | **D** | D1 — API DGI, débit d'un sticker | FIS-06, §11.3 |
| `item_certifie` — persistance des `id` d'items retournés | **D** | D1 — produit par l'API de certification | FIS-06 |
| `file_certification` — transition `EN_ATTENTE → SOUMISE → CERTIFIEE` | **D** | D1 — autorité externe | FIS-05 |
| `file_certification` — état `INDETERMINEE` | **D** | D1 — **jamais rejoué automatiquement** | FIS-05 |
| Rapprochement manuel d'un `INDETERMINEE` | **D** | D1 — décision humaine sur état externe | FIS-05 |
| `compteur_stickers` — décrément, seuil | **D** | D1 — décrément côté DGI | FIS-07 |
| `etat_reversement_communal` — génération | **A** | A4 — rapport dérivé, recalculable | FIS-08 |
| Export comptable | **A** | A4 — dérivé, recalculable | FIS-09 |

> **Aucun document fiscal n'est jamais généré hors ligne.** Le mode dégradé produit un document
> **opérationnel** (§5.5) portant la mention « Document non fiscal — ne tient pas lieu de
> facture », et place l'opération en file de régularisation.

### 5.5 `socle/documents`

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `document_operationnel` — brouillon non numéroté | **A** | A4 — sans unicité, sans effet | FIS-02 |
| `document_operationnel` — **émission avec numéro interne** | **B** | B3 — **numérotation**, explicitement B au cadrage §11.3 | FIS-02, §11.3 |
| `numerotation_document` — allocation d'un numéro de séquence | **B** | B3 — ressource unique par établissement | §11.3 |
| Ticket de commande, bon de préparation, reçu — impression | **A** | A4 — rendu local, file d'impression avec reprise | IMP-01 |
| Note provisoire — génération | **A** | A4 — dérivée de la note, mention non fiscale obligatoire | IMP-02 |
| `modele_document` — en-tête, pied, mentions | **C** | C2 — référentiel de branding | IMP-04 |

### 5.6 `socle/synchronisation`

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `evenement_outbox` — écriture dans la transaction métier | **A** | A4 — append-only, immuable, **rétention illimitée** | TRX-02 |
| `evenement_outbox` — marquage « publié » | **A** | A4 — jamais de suppression | TRX-02 |
| `reconciliation_orpheline` — création de l'élément en file | **A** | A4 — constat append-only | SYN-03 |
| `reconciliation_orpheline` — **résolution** (avoir, prise en charge, rattachement) | **B** | B3 — effet monétaire, **résolution humaine obligatoire** | SYN-03 |
| Horodatage d'autorité — attribution | **serveur uniquement** | — | SYN-04 |
| Horodatage client — enregistrement indicatif | **A** | A4 — ordre d'affichage local, jamais de logique métier | SYN-04 |

> **La file d'actions locale du terminal n'est pas une entité de ce registre** : c'est
> l'infrastructure qui transporte les écritures A. Elle **ne contient jamais** de donnée B, C
> ou D en cache d'écriture (cadrage §11.5 règle 4).

### 5.7 `socle/pilotage`

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| Tableaux de bord, KPI, rapports périodiques | **A** | A4 — lecture dérivée, **fraîcheur affichée** | DIR-01, DIR-02, DIR-05 |
| Consultation du journal d'audit | **A** | A4 — lecture | DIR-04 |
| `alerte_configurable` — seuils de remise, d'écart, de rebascule | **C** | C2 — paramétrage | DIR-04 |

### 5.8 `socle/editeur`

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| Provisionnement de tenant, seeds fiscaux, comptes initiaux | **C** | C2 — relation éditeur–client | ADM-01 |
| `plan`, `palier`, seuils et montants d'abonnement | **C** | C2 — relation éditeur–client | ADM-03 |
| `abonnement` — souscription, gratuité, remise commerciale | **C** | C2 — relation éditeur–client | ADM-03 |
| `unite_facturable` — comptage par la verticale | **C** | C2 — dérivé du référentiel | ADM-03 |
| Encaissement d'abonnement | **D** | D1 — explicitement D au cadrage §11.3 | ADM-04 |
| Webhook de paiement — validation HMAC, idempotence | **D** | D1 — agrégateur | ADM-04 |
| Suspension pour impayé | **C** | C2 — relation éditeur–client | ADM-04 |
| `telemetrie_parc` — version, santé, erreurs | **A** | A4 — append-only | TRX-07, ADM-02 |
| `bundle_diagnostic` — export | **A** | A4 — dérivé | TRX-07, ADM-05 |

### 5.9 `socle/metriques`

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `evenement_metrique` — ingestion par lots | **A** | A4 — append-only, **idempotent par UUID** | MET-02 |
| `agregat_quotidien` | **A** | A4 — dérivé, recalculable | MET-03 |

> La **taxonomie d'événements** (MET-01) est versionnée dans le dépôt, pas en table : elle
> relève du contrat de code, non de ce registre.

---

## 6. `capacites/` — transverses

### 6.1 `capacites/stocks`

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `article_stock` — création, `seuil_alerte`, `unite_mesure` | **C** | C2 — référentiel | STK-01 |
| `point_de_stock` — cave, cuisine, bar | **C** | C2 — référentiel | STK-01 |
| Liaison article de catalogue → article de stock | **C** | C2 — référentiel | STK-01 |
| `mouvement_stock` — entrée, sortie sur vente, ajustement, transfert, casse | **B** ⚠️ | B3 — décrément d'une quantité partagée | STK-02, §11.3 |
| `inventaire` — saisie, écart | **B** | B3 — effet sur les quantités | STK-03 |
| `alerte_seuil` — déclenchement, notification | **A** | A4 — explicitement A au cadrage §11.3 | STK-04 |
| Consultation du stock hors ligne | **A** | A4 — lecture, **toujours affichée comme indicative** | STK-02 |

> ⚠️ **`mouvement_stock` est B par décision par défaut, décision B-05 non tranchée.** Si le
> pilote confirme que le stock sert à **détecter le vol**, il reste B et sérialisé ; s'il ne sert
> qu'à **réapprovisionner**, il peut passer en A. Voir §12, O-02.
> `quantite` est en **`NUMERIC`**, jamais en entier ; `cout_unitaire` est nullable et **jamais
> renseigné au MVP**.

---

## 7. `verticales/hebergement`

### 7.1 Référentiel

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `categorie` — nom, capacité, temps de remise en état par formule | **C** | C2 — référentiel | HEB-01, §11.3 |
| `unite` (spécialisation de `ressource_reservable`) — code, étage | **C** | C2 — référentiel | HEB-01, §11.3 |
| `formule` — type, contraintes, `assujettie_taxe_nuitee`, `regle_conversion_taxe` | **C** | C2 — référentiel fiscal | HEB-03, §11.3 |
| `bareme_palier` — paliers de passage, heure supplémentaire | **C** | C2 — référentiel tarifaire | HEB-04, §11.3 |
| `calendrier_tarifaire` — date d'effet, date de fin | **C** | C2 — référentiel tarifaire | HEB-07 |
| Plages de demi-journée | **C** | C2 — référentiel | HEB-05 |

### 7.2 Occupation et disponibilité

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `occupation` — **attribution d'unité** sur un `tstzrange` | **B** | B3 — ressource unique, contrainte d'exclusion GiST | HEB-02, §11.3 |
| Intervalle de remise en état | **B** | B3 — intégré à l'intervalle d'indisponibilité | HEB-02 |
| `unite.statut_occupation` (libre / occupée / réservée) | **dérivé** | — | HEB-06 |
| `unite.statut_menage` (à nettoyer / propre / maintenance) | **A** | A4 — **dernier-écrit-gagne autorisé, seul cas** | HEB-06, §11.3 |
| **Mise hors service** d'une unité | **B** | B3 — retire une ressource de la disponibilité | HEB-06, §11.3 |
| Forçage de disponibilité (tracé au journal d'audit) | **B** | B3 — contourne une ressource unique | CPT-04 |

> **`unite.statut_occupation` n'est jamais posé à la main.** Il est calculé depuis les
> occupations. Le confondre avec `statut_menage` produit des doubles attributions
> (cadrage §11.4).

### 7.3 Séjour

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `client` — création, modification de fiche | **C** | C2 — partagé entre les établissements du tenant | SEJ-01 |
| `client.preferences`, note interne, photo | **A** | A4 — explicitement A au cadrage §11.3 | SEJ-01, §11.3 |
| Extraction OCR d'une pièce d'identité | **A** | A4 — explicitement A ; **entièrement dégradable** | SEJ-06, §11.3 |
| `sejour` — **check-in**, attribution d'unité | **B** | B3 — ressource unique | SEJ-02, §11.3 |
| `accompagnant` — ajout | **A** | A4 — explicitement A au cadrage §11.3 | SEJ-02, §11.3 |
| `fiche_police` — génération | **B** | B3 — dérivée du check-in, numérotée | SEJ-02 |
| `ligne_sejour` — hébergement, extras | **B** | B3 — effet monétaire sur la note | SEJ-03 |
| `ligne_sejour` — consommation venue d'un point de vente | **classe de la ligne d'origine** | — | §8 |
| **Transfert de charges** entre séjours | **B** | B3 — effet monétaire, tracé | SEJ-03, §11.3 |
| Remise sur la note | **B** | B3 — effet monétaire, journal d'audit | SEJ-03, §11.3 |
| `sejour` — **check-out**, taxe de nuitée **figée** | **B** | B3 — clôt la note, déclenche le document fiscal | SEJ-04, §11.3 |
| **Prolongation** | **B** | B3 — étend l'intervalle, conflit possible | SEJ-04, §11.3 |
| **Départ anticipé** — recalcul, régularisation | **B** | B3 — effet monétaire | SEJ-04, §11.3 |
| **Changement d'unité** en cours de séjour | **B** | B3 — deux intervalles, ressource unique | SEJ-04, §11.3 |
| **Rebascule de palier de passage** | **B** | B3 — effet monétaire, journal d'audit | HEB-04, §11.3 |
| Bascule passage → nuitée au-delà du seuil | **B** | B3 — effet monétaire | HEB-04 |
| Vente à un **client extérieur** (sans hébergement) | **B** | B3 — encaissement immédiat | SEJ-05 |

> **Le calcul de durée de passage s'appuie exclusivement sur l'horodatage d'autorité.** En mode
> C, le nœud de site fait autorité ; **jamais le terminal** (cadrage §11.4).

### 7.4 Réservation

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `reservation` — création, modification | **B** | B3 — ressource unique sur un intervalle | RSV-01, §11.3 |
| `reservation` — expiration d'une provisoire | **B** | B3 — libère une ressource | RSV-01 |
| `arrhes` — encaissement | **classe du mode** (§5.3) | — | RSV-03, §11.3 |
| Politique d'annulation — paramètres | **C** | C2 — paramétrage | RSV-03 |
| **Annulation** — libération de l'intervalle | **B** | B3 — ressource unique, effet monétaire | RSV-04, §11.3 |
| **No-show** — facturation selon politique | **B** | B3 — effet monétaire | RSV-04, §11.3 |
| Conversion réservation → séjour | **B** | B3 — attribution d'unité | RSV-05 |

### 7.5 Maintenance et salle de réunion

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `incident_maintenance` — signalement | **A** | A4 — explicitement A au cadrage §11.3 | §11.3 |
| `intervention` — compte rendu | **A** | A4 — explicitement A au cadrage §11.3 | §11.3 |
| Réservation de salle de réunion | **B** | B3 — `SALLE_REUNION` est une spécialisation d'hébergement | PDV-08, HEB-05 |

---

## 8. `verticales/restauration`, `verticales/bar`, `verticales/pressing`

### 8.1 Catalogue

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `article` — nom, prix, `taux_tva`, `unite_mesure`, `suivi_stock` | **C** | C2 — catalogue et prix, explicitement C au cadrage §11.3 | PDV-01, §11.3 |
| `article.code_barre`, `.article_parent_id` | **C** | C2 — référentiel (nullables, **non utilisés au MVP**) | PDV-01 |
| Catégorie d'affichage, ordre | **C** | C2 — référentiel | PDV-01 |
| Modification de tarif | **C** | C2 — référentiel, journal d'audit | PDV-01, CPT-04 |

> **Le prix est verrouillé à la création de la ligne de commande.** Une modification de tarif
> ultérieure ne modifie aucune commande existante.

### 8.2 Commande — le cœur du besoin hors-ligne

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `commande` — ouverture | **A** | A4 — sans unicité tant qu'elle n'est pas numérotée | PDV-03 |
| `ligne_commande` — **ajout**, quantité (`NUMERIC`), commentaire | **A** | A4 — explicitement A au cadrage §11.3 | PDV-03, §11.3 |
| `ligne_commande` — modification **avant envoi** | **A** | A4 — purement locale, **jamais synchronisée avant envoi** | PDV-03, §11.3 |
| **Envoi en préparation** (cuisine, bar, pressing) | **A** | A4 — explicitement A au cadrage §11.3 | PDV-04, §11.3 |
| Marquage « servi », marquage « prêt » | **A** | A4 — explicitement A au cadrage §11.3 | PDV-04, §11.3 |
| **Annulation d'une ligne envoyée** | **B** | B3 — motif obligatoire, journal d'audit | PDV-03, §11.3 |
| **Remise** sur une ligne ou une addition | **B** | B3 — effet monétaire, permission, audit | PDV-03, §11.3 |
| Cible de facturation (`table`/`sejour`/`comptoir`/`emporter`) | **attribut de la commande** | — | PDV-02 |

### 8.3 Addition de table

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| **Ouverture** d'une table | **B** | B3 — ressource unique, explicitement B au cadrage §11.3 | PDV-02, §11.3 |
| **Fermeture** d'une table | **B** | B3 — effet monétaire | PDV-02, §11.3 |
| **Transfert** entre tables, **fusion** | **B** | B3 — ressource unique | PDV-02, §11.3 |
| **Division d'addition** — par ligne ou par montant | **B** | B3 — effet monétaire, cibles multiples | PDV-05, §11.3 |

### 8.4 Pressing

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `bon_de_depot` — création avec **numéro de retrait** | **B** | B3 — numérotation, ressource unique | PDV-06 |
| Liste d'articles déposés, état constaté | **A** | A4 — append-only, rattaché au bon | PDV-06 |
| Transition `depose → en_traitement → pret` | **A** | A4 — sans effet monétaire, sans unicité | PDV-06 |
| Transition `pret → retire` (avec règlement) | **B** | B3 — effet monétaire, clôt le bon | PDV-06 |
| Rattachement d'un bon au séjour d'un client logé | **B** | B3 — effet monétaire sur la note | PDV-06 |

### 8.5 Commande par QR

*Crate d'accueil à confirmer — voir §12, O-03.*

| Entité ou opération | Classe | Branche | Réf. |
|---|---|---|---|
| `jeton_table` — génération, **révocation** | **C** | C2 — explicitement C au cadrage §11.3 | QRC-01, §11.3 |
| Panier client sur la page publique | **hors registre** | — surface web publique, hors application | QRC-02 |
| **Réception** d'une commande QR en état `À_CONFIRMER` | **A** | A4 — explicitement A au cadrage §11.3 | QRC-03, §11.3 |
| **Validation par le personnel** | **B** | B3 — explicitement B au cadrage §11.3 | QRC-03, §11.3 |
| Limitation de débit par jeton | **éphémère** | — compteur Redis, reconstructible | QRC-04 |

---

## 9. Ce qui n'est pas classé

| Élément | Pourquoi |
|---|---|
| Sessions, JWT, refresh | Éphémère Redis reconstructible (constitution, principe II) |
| File de certification FNE en Redis | Éphémère ; l'état durable vit en Postgres (§5.4) |
| Verrous distribués, limitation de débit, cache de catalogue | Éphémère reconstructible |
| File d'actions locale du terminal | Infrastructure de transport, jamais un cache d'écriture B/C/D |
| Taxonomie d'événements, registre des traitements ARTCI | Versionnés dans le dépôt, pas en table |
| Panier de la page publique QR | Surface web publique hors application |

---

## 10. Provisions — tables sans logique au MVP

Ces entités ont une **classe déclarée d'avance** afin qu'aucune implémentation future ne
reparte d'une page blanche. **Aucune UI, aucune logique au MVP** (constitution, principe X).

| Entité | Classe prévue | Branche | Réf. |
|---|---|---|---|
| `mapping_comptable`, `exercice_comptable` | **C** | C2 — référentiel comptable du tenant | TRX-02b |
| `employe` | **C** | C2 — référentiel RH ; **jamais confondu avec `compte`** | CPT-00 |
| `partenaire` (`tenant_id` **nullable**), `demande_partenaire` | **C** | C2 — référentiel, éventuellement inter-tenant | ETB-07 |
| `compte_compensation`, `mouvement_compensation` | **B** | B3 — effet monétaire | ETB-07 |
| `convention_inter_etablissements` | **C** | C2 — relation entre deux tenants | cadrage §4.3 |
| Modules additionnels (`SPA`, `BOULANGERIE`, `SUPERETTE`, `QUINCAILLERIE`, `EXCURSION`) | **C** | C2 — référentiel | ETB-08 |
| Capacités non implémentées (`LIVRAISON`, `PRODUCTION`, `COMMERCE_EN_LIGNE`, `FIDELITE`, `DEVIS`, `COMPTES_CLIENTS`) | **C** | C2 — référentiel ; **refus explicite au MVP** | ETB-02b |
| Profils de stock `VALORISE`, `DETAILLE` | **C** | C2 — référentiel ; **refus explicite au MVP** | ETB-02b |
| `mouvement_stock.cout_unitaire` | **B** | B3 — suit `mouvement_stock` ; **jamais renseigné au MVP** | STK-02 |
| `contrat_location`, `caution`, `charge_locative`, `etat_des_lieux` | **C** | C2 — référentiel contractuel | HEB-08 |
| `prestation_incluse` | **C** | C2 — référentiel attaché à la formule | HEB-09 |
| Décompte d'une prestation incluse *(incrément 2)* | **B** | B3 — décrément d'un quota | HEB-09 |
| `devis`, `document_commercial` | **B** | B3 — numérotation propre | FIS-11 |
| `EmissionChannel::Terne`, `ligne_facture.rne_ref` | **D** | D1 — canal fiscal externe | FIS-10 |
| `compte_client`, `encours`, `condition_reglement` | **B** | B3 — effet monétaire par établissement | CAI-07 |
| `dispositif`, `AccessController` | **A** | A4 — **canal hors ligne obligatoire** : code à usage unique validable sans réseau | cadrage §14.21 |

> **La contrainte du contrôle d'accès est à respecter dès maintenant** : tout mécanisme
> d'ouverture d'unité devra disposer d'un canal hors ligne. Une porte qui ne s'ouvre pas parce
> que le réseau est tombé est un incident grave.

---

## 11. Tests obligatoires par classe

Repris de `docs/user-stories-v1.md` §0.7. Ces tests font partie de la story qui introduit
l'entité, pas d'un lot de rattrapage.

| Classe | Tests exigés |
|---|---|
| **A** | **Rejeu** — la même écriture envoyée trois fois produit un seul enregistrement. **Désordre** — trois écritures appliquées dans les six ordres possibles produisent le même état final. |
| **B** | Test qui **échoue si l'opération est atteignable depuis un chemin de code exécutable hors ligne**. Test de concurrence : deux exécutions simultanées, une seule réussit. |
| **C** | Test qui **échoue si l'opération est atteignable depuis un chemin de code exécutable hors ligne**. Test d'isolation multi-tenant sur l'endpoint. |
| **D** | Test qui **échoue si l'opération est atteignable depuis un chemin de code exécutable hors ligne**. Test de **double soumission au retour du réseau**. |
| **Toute entité rattachée à un séjour** | Test du **scénario orphelin** (SYN-03). |

**Deux tests transverses permanents :**

- **Réseau coupé puis rétabli** au milieu d'une journée d'exploitation simulée — la clôture
  journalière tombe **au franc près** (SYN-04).
- **Agnosticité du socle** — un établissement portant un module fictif minimal, sans aucune
  capacité, va de la création à la clôture journalière (ETB-02c).

---

## 12. Cas pièges et décisions ouvertes

### Cas pièges traités explicitement

1. **Le statut d'unité n'est pas une donnée libre.** « Occupée » et « réservée » sont **dérivés**
   des occupations. Seul `statut_menage` est librement modifiable, en A. Les confondre produit
   des doubles attributions.
2. **L'écriture orpheline est le conflit le plus fréquent.** Une consommation saisie hors ligne
   arrive sur un séjour clos et facturé → **file de réconciliation à résolution humaine
   obligatoire**, jamais de rejet silencieux ni d'ajout d'office. Aggravé par l'avoir FNE par
   quantité. **Écran testé en priorité.**
3. **Les horloges des terminaux ne sont pas fiables.** Horodatage client indicatif, horodatage
   d'autorité pour **toute** logique métier, fiscale, de clôture et de durée de passage. Alerte
   au-delà de 5 minutes de dérive.
4. **Le passage aggrave la sensibilité à l'horloge.** Le début d'occupation est posé par le
   serveur au check-in ; en mode C, par le nœud de site. **Jamais par le terminal.**
5. **iOS n'a pas de synchronisation en arrière-plan.** La file se vide **au retour au premier
   plan par défaut** sur toutes les plateformes.

### Décisions ouvertes

| # | Décision | Effet si tranchée autrement | Échéance |
|---|---|---|---|
| **O-01** | **`client` / `personne` en C** rend le check-in d'un **client inconnu** impossible hors ligne, y compris en mode C. Options : (a) maintenir C et exiger le réseau pour une fiche nouvelle ; (b) descendre `client` en B avec unicité par établissement et fusion au cloud ; (c) accepter un « client provisoire » local de classe A, promu en C à la synchronisation. | Option (a) = friction au comptoir en coupure ; (b) = doublons inter-établissements ; (c) = complexité de promotion | Avant SEJ-02 (tranche T1) |
| **O-02** | **`mouvement_stock` en A ou en B** — décision B-05 du cadrage. Si le stock sert à détecter le vol, il reste B ; s'il ne sert qu'à réapprovisionner, il passe en A. | A = saisie hors ligne possible, tout se simplifie ; B = sérialisation stricte | S4, avec le pilote (avant tranche T5) |
| **O-03** | **Crate d'accueil de la surface QR.** Le principe II de la constitution ne liste que `hebergement`, `restauration`, `bar`, `pressing` dans `verticales/`. La commande QR est transverse à `restauration` et `bar`. | Un crate `capacites/` dédié, ou un partage entre les deux verticales | Avant QRC-01 (tranche T4) |

> Les décisions ouvertes n'autorisent aucun contournement : jusqu'à leur arbitrage, **la classe
> inscrite dans ce registre s'applique** — c'est toujours la plus stricte des options.

---

## 13. Journal des modifications

| Version | Date | Modification |
|---|---|---|
| 1.0.0 | 2026-07-30 | Création. Classement initial de toutes les entités des modules TRX, ETB, CPT, HEB, SEJ, RSV, PDV, QRC, CAI, FIS, SYN, IMP, STK, DIR, ADM, MET, plus les provisions du cadrage §14. Dérivé de `docs/cadrage-v1.md` §11 et `docs/user-stories-v1.md` §0.7. Trois décisions ouvertes consignées (O-01, O-02, O-03). |
| 1.0.2 | 2026-07-31 | **`profil_stock` et `parametre_catalogue` ajoutées au §5.1, classe C** — les deux référentiels globaux que le cycle 002 crée et que le registre ne nommait pas. `profil_stock` n'existait qu'en tant que colonne dans la ligne de `module_capacite` ; devenue table (research.md R-03 : ouvrir un profil est une écriture de configuration, pas une migration), elle doit s'y déclarer pour elle-même. **Ajout d'une ligne de portée générale : la LECTURE EN CACHE de tout référentiel est de classe A, avec fraîcheur affichée**, quand son écriture reste C. Le registre classe des opérations, pas des tables — sans cette distinction écrite, un cycle ultérieur aurait conclu qu'un référentiel de classe C ne se lit pas hors ligne, ce qui rendrait le produit inutilisable dès la première coupure. Le mécanisme de cache relève de SYN-01/02 et d'ETB-06 ; seule la classe est arrêtée ici. |
| 1.0.1 | 2026-07-31 | **`note_etablissement` ajoutée au §5.1, classe A, branche A4** — entité du module doré du cycle 001 (TRX-01). Append-only : ni `UPDATE` ni `DELETE` n'est accordé à `kaya_app`, une correction est une nouvelle note. Ses deux tests de classe A vivent dans `backend/tests/note_etablissement_classe_a.rs` et sont exécutés en intégration continue. À partir de ce cycle, le registre n'est plus seulement documentaire : `backend/tests/classes_offline.rs` compare les tables réelles aux entités déclarées ici et **fait échouer le build** sur toute table absente. Le sens de comparaison est table → registre : une entité déclarée mais pas encore implémentée est normale, une table non déclarée est l'erreur à attraper. |
