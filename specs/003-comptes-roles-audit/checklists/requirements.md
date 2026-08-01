# Specification Quality Checklist: Comptes, rôles cumulables et journal d'audit

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-08-01
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

**Itération 1 — trois écarts corrigés avant clôture :**

1. *Détails d'implémentation dans les exigences d'authentification.* La rédaction initiale
   nommait « JWT », « refresh token » et « Redis ». Reformulé en « accès de courte durée »,
   « moyen de rafraîchissement révocable » et « données éphémères et reconstructibles »
   (FR-009, FR-015). Le choix technique reste tranché par les documents de référence
   (`registre-classes-offline.md` §9) et sera repris tel quel en `/speckit-plan`.
2. *Critère de succès non mesurable.* « Les messages d'erreur ne révèlent pas si un compte
   existe » ne se teste pas en l'état ; SC-002 fixe désormais message, code **et** ordre de
   grandeur du temps de réponse sur 100 tentatives de chaque type.
3. *Périmètre du journal d'audit ambigu face à DIR-04.* La frontière est écrite deux fois —
   FR-040 (exigence négative) et § Out of Scope (renvoi vers T5).

**Trois questions posées et tranchées le 2026-08-01** (§ Clarifications) :

- **Q1 — écran de connexion absent de la matrice de dérivation.** Blocage réel de la porte
  **P-19** : aucun des 42 écrans n'est un écran de connexion, et CPT-01 en exige un.
  → **Tranché : ajouter `R0` Connexion, hérité de `G2`.** L'amendement de
  `docs/design/derivation.md` est une **tâche de ce cycle, faite avant que l'écran ne soit
  codé** — c'est la seule modification d'un document normatif que le cycle emporte.
- **Q2 — OTP SMS au MVP.** → **Tranché : mot de passe fort seul, `OTP_SMS` refusée
  explicitement** (FR-008), sur le patron du refus des capacités non implémentées (P-06).
- **Q3 — périmètre du catalogue de permissions.** → **Tranché : permissions des modules livrés
  seulement**, référentiel extensible sans migration, FR-021 refusant toute permission gardant
  zéro action.

Aucune question ne reste ouverte : la spec est prête pour `/speckit-plan`.

**Vérification de non-régression documentaire** : la spec ne contredit aucune source de
préséance supérieure. Les neuf lignes de `docs/registre-classes-offline.md` §5.2 sont reprises
sans altération, y compris le classement **A** de `journal_audit` et son encadré (« l'opération
tracée garde sa propre classe »), et le statut non classé des sessions au §9.
