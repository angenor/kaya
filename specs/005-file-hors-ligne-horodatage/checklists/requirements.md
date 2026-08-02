# Specification Quality Checklist: Classification hors-ligne, file d'actions et horodatage d'autorité

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-08-02
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

**Trois passes de validation ont été nécessaires.** Ce qui a été corrigé :

1. **Passe 1 — implémentation qui fuyait.** Six exigences nommaient des fichiers réels
   (`app/core/sync/vidage.ts`), des mécanismes (`IndexedDB`, symbole TypeScript unique) et des
   noms de tables. Réécrites en termes de comportement observable : « la file MUST être portée par
   un point de sortie unique » plutôt que « `viderFile()` MUST rester la seule sortie ». Les noms
   de fichiers subsistent **uniquement** dans la section « Contexte et traçabilité », qui est un
   inventaire de l'existant et non une exigence — le distinguer était le point.

2. **Passe 2 — deux exigences non testables.** « L'indicateur est lisible d'un coup d'œil » et
   « la file est persistante » ne se vérifiaient pas. La première a reçu son critère (SC-005,
   moins de deux secondes sans cliquer) ; la seconde son scénario (extinction complète du
   terminal, scénario 3 de la story 1). L'état « dégradé » a reçu une définition opérationnelle —
   sans elle, aucun test ne pouvait distinguer dégradé de hors ligne.

3. **Passe 3 — un critère de succès rédigé en termes techniques.** « Le serveur déduplique par
   UUID v7 » disait le mécanisme, pas le résultat. Devenu SC-003 : « la même écriture soumise
   trois fois produit un seul enregistrement et trois réponses identiques ». Le mécanisme reste
   en FR-018, où il est à sa place — c'est une exigence, pas une mesure de succès.

**Quatre décisions de portée sont tranchées et consignées** au § Clarifications de la spec
(session 2026-08-02) :

- **Premier passager de la file** → l'écran minimal de note interne est livré avec le mécanisme.
  La file a donc un passager réel, exerçable en navigateur, plutôt qu'un mécanisme complet que
  personne n'emprunte avant la tranche T2.
- **Portée de l'invariante hors-ligne côté application** → les **deux versants** (FR-005b,
  FR-005c) : contrainte de type **et** balayage des écrans en direct, réseau coupé, avec périmètre
  découvert et nombre d'écrans parcourus rapporté (SC-006).
- **Déduplication serveur** → **pas une question ouverte**. Le patron du cycle 001 s'applique
  (FR-018b à FR-018d) : UUID client en clé primaire, *créée* / *déjà présente*, jamais de conflit,
  **aucun événement au rejeu**. Sa limite — une ligne supprimée serait recréée — est **nommée sans
  être traitée**, parce qu'elle ne mord que sur une classe B en mode nœud de site, qui n'existe
  pas. Cette spec avait à tort inscrit le sujet comme « à creuser au plan ».
- **Périmètre des portes** → le correctif dépasse la porte du registre (FR-004b à FR-004d) : un
  module d'énumération partagé, dont les 21 chemins de crates écrits en dur sur six fichiers sont
  ramenés à zéro (SC-007b).

**Limite assumée, écrite plutôt qu'enfouie** : la porte de FR-005 (« aucune opération B, C ou D
atteignable hors ligne ») vérifie que *la question a été posée*, jamais que *la réponse est
juste*. Aucune lecture du schéma ne peut retrouver qu'un encaissement est B en espèces et D en
Mobile Money : c'est métier. La justesse des classes reste humaine et revue mensuellement — même
limite que la porte du registre côté serveur, et pour la même raison. Prétendre l'automatiser
produirait un vert qui empêche la relecture.
