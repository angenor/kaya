# Specification Quality Checklist: Socle technique du monorepo Kaya

**Purpose**: Valider la complétude et la qualité de la spécification avant de passer à la planification
**Created**: 2026-07-30
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs) — *avec réserve, voir Notes §1*
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders — *avec réserve, voir Notes §1*
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification — *avec réserve, voir Notes §1*

## Notes

### 1. Réserve assumée sur l'absence de détails d'implémentation

Trois items sont cochés **avec réserve explicite**, et c'est un choix, pas un oubli.

Cette fonctionnalité **est** de l'infrastructure : son livrable est une arborescence de code, un
contrat d'API, un schéma de base et une chaîne d'intégration continue. La consigne du cycle est
de *« reprendre les critères d'acceptation de TRX-01 à TRX-05 tels quels, sans inventer
d'exigences supplémentaires »* — or ces critères nomment eux-mêmes des artefacts techniques :
`/api-docs/openapi.json`, `/health`, `tenant_id`, sécurité au niveau ligne activée et forcée,
`{type, agrégat, tenant_id, etablissement_id, payload, horodatage}`, noms de crates et de tables.
Les paraphraser aurait introduit une divergence avec `docs/user-stories-v1.md`, qui fait foi
(constitution, hiérarchie documentaire).

**Atténuation appliquée** : partout où le nom d'un outil n'était pas exigé par un critère
d'acceptation, il est remplacé par son rôle — « bibliothèque d'accès aux données », « cadre
web », « éditeur de liens rapide », « cache de compilation partagé », « stockage objet », « cache
éphémère », « service de suivi des erreurs ». Les versions exactes vivent dans
`docs/versions-gelees.md`, jamais dans cette spécification. Les identifiants de tables, de
colonnes et de traits sont conservés **littéralement**, conformément à la règle de nommage du
projet : les traduire ou les normaliser créerait une divergence entre le code et les documents de
référence.

**Lecteur visé** : l'Admin éditeur, seul persona de ce cycle — c'est-à-dire le développeur solo
lui-même. Il n'existe pas de partie prenante non technique pour un cycle d'infrastructure.

### 2. Trois tâches obligatoires — état réel constaté

Deux des trois « tâches obligatoires » du prompt portaient sur des artefacts **déjà produits** au
moment de la spécification. C'est consigné dans les hypothèses 2 et 3 de la spec plutôt que
silencieusement reformulé :

| Tâche | État constaté | Ce que le cycle livre |
|---|---|---|
| Vérifier et épingler les 10 versions | `docs/versions-gelees.md` v1.0.2, vérifié le **2026-07-30** (jour du cycle), 10 briques, URL de registre citée pour chacune | Épinglage exact dans les manifestes + lockfiles + porte P-20. Revérification seulement si le gel dépasse un mois ou si une brique change |
| Écrire le module doré | Non commencé | Livrable intégral (US2, FR-023 à FR-028) |
| Créer le registre des classes hors-ligne | `docs/registre-classes-offline.md` existe depuis le **2026-07-30**, 4 classes, arbre de décision, classement par crate | La **porte de CI** qui le rend opposable + les tests par classe (US8, FR-066 à FR-070) |

### 3. Décisions ouvertes non bloquantes pour ce cycle

- **O-01** (`client` / `personne` en classe C) — à trancher avant SEJ-02, sans effet sur le socle.
- **O-02** (classe de `mouvement_stock`) — à trancher avec le pilote, tranche T5.
- **O-03** (crate d'accueil de la surface QR) — à trancher avant QRC-01.
- **B-02** (traitement fiscal du passage et de la demi-journée) — paramètre par formule, aucune
  valeur en dur ; sans objet tant qu'aucune règle fiscale n'est écrite.
- **Confirmation du choix de version de la bibliothèque d'accès aux données** par le spike sur les
  contraintes d'exclusion et les intervalles de temps (cadrage §16) — **seul point du gel resté
  ouvert**, et le seul qui pourrait rétroagir sur le module doré. À traiter tôt en
  `/speckit-plan`.

### 4. Résultat de validation

Validation exécutée en **une itération**. Aucun item en échec, aucun marqueur
`[NEEDS CLARIFICATION]` restant. Les trois réserves du §1 sont assumées et documentées, non
corrigibles sans contredire une source de vérité de rang supérieur.

**Prête pour `/speckit-plan`.**
