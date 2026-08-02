# Specification Quality Checklist: Unités louables, formules de location et moteur de disponibilité

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

### Itération 1 — un point corrigé, deux points assumés et explicités

1. **SC-004 était formulé en temps de réponse d'API.** Corrigé : il énonce ce que Yao constate —
   aucune attente perceptible — et rattache le chiffre à la cible de trente secondes de SEJ-02,
   qui est une exigence d'usage documentée.

2. **Le vocabulaire technique du schéma est conservé, délibérément.** Les mots « table »,
   « colonne », « endpoint » et « crate » figurent dans plusieurs exigences. Ce n'est pas une
   fuite : dans ce projet, la **structure du schéma est elle-même une exigence** — le principe II
   impose un schéma par module et une hiérarchie de crates, le registre des classes hors-ligne
   classe des entités, et les provisions se définissent précisément comme « la table existe, la
   logique n'existe pas ». Une exigence qui dirait « le système prépare les prestations incluses »
   sans dire « la table est créée, vide » ne serait pas vérifiable. Même choix qu'aux cycles 002
   et 003.

   En revanche, le corps normatif **ne nomme ni le moteur de base de données, ni la syntaxe de la
   contrainte d'exclusion, ni le type d'intervalle** : les exigences énoncent « contrainte
   d'exclusion de la base de données » et « intervalle en timestamp avec fuseau ». La formulation
   technique n'apparaît que dans le champ **Input** (verbatim de la demande) et dans la
   § Contexte, où elle cite les documents de référence — traçabilité, non prescription. Une
   spécification qui tairait entièrement la contrainte d'exclusion masquerait la seule décision
   irréversible du cycle, que le principe IV impose nommément.

3. **Trois zones sous-spécifiées, résolues sans marqueur.** Elles auraient produit trois
   [NEEDS CLARIFICATION] ; chacune avait une réponse dérivable des sources de vérité, et a donc
   été tranchée puis inscrite en § Clarifications :
   - **écran du référentiel des catégories et unités** → aucun, la matrice de dérivation ne
     l'inscrit pas et sa règle de conduite interdit de le coder (FR-041) ;
   - **attribution d'occupation exposée en API sans check-in** → oui, le test obligatoire de
     classe B et la permission annoncée par le cycle 003 l'exigent tous deux ;
   - **règle de conversion fiscale par défaut** → aucune valeur, l'absence est explicite et
     bloquante (FR-030, FR-031), la décision B-02 n'étant pas tranchée.

### Points de vigilance transmis à `/speckit-plan`

- **P-09 reçoit sa première cible.** La section « Couverture des portes » de la constitution
  exige d'établir le **périmètre** d'une porte, pas seulement sa capacité à échouer : le plan doit
  prévoir le décompte des tables réellement inspectées, faute de quoi la porte resterait
  indistinguable d'une porte à cible vide — exactement le défaut des cycles 001 et 002.
- **Premières permissions rattachées à un module d'activité.** Le filtrage par `module_code` n'a
  jamais été exercé sur une valeur non nulle.
- **Le seul écran du cycle est maquetté**, donc soumis à la maquette et non à un motif dérivé.
- **SC-002 est une exigence de conception, pas seulement de test** : la garantie doit survivre au
  retrait de toute vérification préalable en lecture.
