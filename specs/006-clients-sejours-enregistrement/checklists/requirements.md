# Specification Quality Checklist: Fiches clients, arrivée, départ et prolongation

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-08-03
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

### Itération 2 — 2026-08-03 · **16 / 16**

Les trois marqueurs de l'itération 1 sont levés par l'arbitrage du 2026-08-03.

| Décision | Arbitrage | Effet sur la spécification |
|---|---|---|
| **O-01** (Q1) | Option **(a)** — `client` reste en classe **C** | FR-011 réécrit. Le modèle ne change pas ; la friction en mode nœud de site est **acceptée et nommée**. Une ligne ajoutée à « Out of Scope ». |
| **B-10** (Q2) | **La taxe de séjour est due par SÉJOUR, pas par personne** | FR-018, FR-020, FR-062, FR-064 réécrits ; entités `accompagnant` et `assiette_taxe_sejour_figee` corrigées ; scénario 4 de la story 3 corrigé ; une hypothèse remplacée. **Pas de motif d'exonération** — absence consignée comme décision. |
| **Fiche de police** (Q3) | Option **(a)** — registre minimal, gabarit officiel différé | FR-049 réécrit avec la liste des champs ; une ligne ajoutée à « Out of Scope ». |

### ⚠️ Réserve bloquante pour `/speckit-plan` — amendements documentaires dus

**La décision B-10 contredit trois écrits de rang supérieur à cette spécification** (cadrage §9.6,
FIS-03, FIS-08), plus trois passages dérivés (annexe B, récapitulatif des paramètres, lexique
v1.5.1). Ils sont listés un par un en **§ Suites documentaires dues** de la spécification.

**Tant qu'ils ne sont pas amendés, la spécification est correcte mais isolée** : le cycle **FIS**
(tranche T3) re-dériverait la règle inverse depuis une source qui prime sur elle. C'est précisément
la situation que l'ordre de préséance de la constitution existe pour empêcher. Ces amendements ne
relèvent pas de l'implémentation de ce cycle et **ne bloquent pas** l'écriture du plan technique —
ils bloquent la **cohérence du corpus**, et doivent tomber avant que FIS ne soit spécifié.

**Point de vigilance annexe** : le récapitulatif des paramètres décrit la règle de Deloria comme
« 500 F pour un séjour de 3 nuits », tandis que l'arbitrage raisonne sur « 500 F par nuit ». Les
deux portent sur l'**axe des nuits**, que B-10 ne touche pas, mais ils divergent. Si la pratique est
une taxe par nuit, c'est le **seed** de `regle_conversion_taxe` qui passe à `au_prorata` — un
changement de donnée, pas de code.

### Ce qui a été vérifié par ailleurs

- **Détails d'implémentation** — la spécification nomme des entités et des paramètres
  (`assujettie_taxe_nuitee`, `heure_arrivee_standard`, `heb.unite.attribuer`) parce que ce sont les
  **noms du corpus**, dont la convention de nommage métier en français est une règle de projet
  opposable (`CLAUDE.md`, « Reprendre littéralement les noms des documents »). Aucune structure de
  code, aucun schéma de table, aucun point d'entrée HTTP n'est prescrit.
- **Testabilité des cibles de temps** — la contrainte « moins de 30 s / moins de 60 s / 90 s =
  échec » est traduite en **deux critères distincts** : déterministe et gardé en CI (SC-001 budget
  de gestes et d'appels réseau ; SC-004 budget de temps machine), et chronométré au terrain puis
  consigné (SC-002, SC-003, FR-106). La leçon SC-004 du cycle 004 — *une assertion de temps humain
  en CI rougit au hasard et finit désactivée* — est reprise explicitement plutôt que contournée.
- **Frontières de périmètre** — « Out of Scope » dit pour chacune de ses douze lignes ce qui est
  **exposé** et ce qui ne l'est **pas**, y compris la dette nommée du consentement tracé (TRX-06,
  P1).
- **Tension maquette / registre** — l'état hors ligne de `R4` est tranché par l'ordre de préséance
  (registre, rang 4, prime sur la maquette, rang 7), non par interprétation.

**Statut** : spécification complète, cohérente et **prête pour `/speckit-plan`**.
