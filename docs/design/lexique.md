# Kaya — Lexique du vocabulaire utilisateur

*Source de vérité du vocabulaire visible par l'utilisateur. Extrait de `docs/Kaya_Design.md` §6
le 2026-07-30 — ce fichier fait foi, `Kaya_Design.md` y renvoie.*

**Version 1.0.0**

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
| Unité louable | « Chambre » en hôtel, « logement » en résidence, « salle » pour la réunion — selon le contexte |
| RBAC, permissions | « Ce que chacun peut faire » |
| Synchronisation | « Enregistré » / « En attente d'envoi (4) » / « Hors connexion » |
| Attestation d'intégrité, enrôlement | « Téléphones autorisés » |
| `note_etablissement` | « **Note interne** » / *Internal note* — jamais « note d'établissement » : le §6 pose déjà que l'utilisateur est toujours dans le sien, le mot serait superflu sur un bouton |

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

## Voir aussi

- `docs/design/derivation.md` — de quel motif maquetté hérite chaque écran non maquetté
- `docs/Kaya_Design.md` §5 « Les neuf règles » — dont la règle 6, « zéro jargon »
- `docs/design/composants.md` — les 14 composants canoniques
