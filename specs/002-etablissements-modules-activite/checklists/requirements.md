# Specification Quality Checklist: Tenants, établissements, modules d'activité et configuration héritée

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-07-31
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
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
- [x] No implementation details leak into specification

## Notes

**Itération 1 — 2026-07-31.** Quatre points relevés et corrigés avant clôture :

1. *Marqueurs de clarification* — deux questions de périmètre engageaient matériellement le
   cycle (portée du bout-en-bout des trois tests structurels ; périmètre d'écrans). Posées et
   tranchées avant rédaction plutôt que laissées en marqueurs. Réponses consignées en
   § Clarifications.
2. *Critères non mesurables* — trois critères de succès initialement qualitatifs (« les tests
   passent », « l'interface est propre ») réécrits en comptages vérifiables : SC-002 (étapes
   exercées comparées au total déclaré), SC-004 (neuf cas de refus), SC-006 (couverture de la
   matrice de résolution).
3. *Cas limites implicites* — onze cas limites explicités avec leur réponse, dont trois qui
   n'apparaissent dans aucune source et se seraient découverts en implémentation : désactivation
   d'un service portant des opérations, modification du fuseau après horodatage, modification de
   la devise après une opération financière.
4. *Périmètre des provisions* — ETB-07 et ETB-08 sont « tables seulement » au MVP, ce qui ne dit
   pas *quel cycle* crée ces tables. Tranché explicitement : **pas celui-ci** (FR-079, § Out of
   Scope).

**Réserve assumée sur « No implementation details ».** La spécification nomme des tables, des
migrations, des crates, des politiques de sécurité au niveau ligne et des portes d'intégration
continue. Ce ne sont pas des choix d'implémentation anticipés : ce sont des **invariants imposés
par les sources de vérité du projet** — constitution (principes I à XII, portes P-01 à P-20),
`docs/registre-classes-offline.md` et `docs/user-stories-v1.md`, dont les critères d'acceptation
sont repris tels quels. Les nommer est la seule façon de rendre les exigences traçables et
opposables. Les choix réellement ouverts — schéma exact, forme du trait de résolution, format et
taille du logo, mécanisme de détection des étapes dues — sont explicitement renvoyés à
`/speckit-plan`. Même arbitrage qu'au cycle 001.

**Prêt pour la suite.** Les huit décisions par défaut sont consignées en § Assumptions et
révisables. Aucune ne bloque `/speckit-plan`.
