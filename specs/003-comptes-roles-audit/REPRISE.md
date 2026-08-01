# Reprise du cycle 003 (CPT) — état au 2026-08-01

*Phases 1 et 2 livrées, T001 à T020. Reste T021 à T064.*

---

## Où en est le cycle

**20 tâches sur 64**, en 21 commits sur `consolidation-polices-styleguide-et-worker` (la spec
pose explicitement qu'aucune branche dédiée n'est créée, comme aux cycles 001 et 002).

**145 tests backend passent**, 0 échec. Portes vérifiées : P-02, P-04, P-05b, P-07, P-10, P-20.

### Ce qui existe désormais

| Livré | Où |
|---|---|
| `R0` à la matrice de dérivation (32 écrans dérivés, 43 au total) | `docs/design/derivation.md` 1.2.0 |
| 18 entrées de vocabulaire CPT | `docs/design/lexique.md` 1.2.0 |
| Taxonomie d'audit, **0/10 branchés** | `docs/taxonomie-audit.md` + `backend/tests/audit_taxonomie.rs` |
| Six migrations `0014` → `0019`, dix tables | `backend/migrations/` |
| `KAYA_JWT_CLE`, `KAYA_SEEDS_MOT_DE_PASSE`, `KAYA_ENVIRONNEMENT` | `backend/api/src/secrets.rs` |
| Liste de 97 747 mots de passe compromis, embarquée | `backend/crates/socle/comptes/src/authentification/` |
| Hachage Argon2id aux paramètres OWASP | `authentification/argon2.rs` |
| Trait `JournalAudit` + validation monétaire JSONB | `audit/{modele,service,taxonomie}.rs` |
| Trois comptes du pilote, Adjoua avec ses trois rôles | `backend/api/src/bin/seeds.rs` |

### Trois écarts au plan, assumés et consignés

1. **`jsonwebtoken` a exigé une feature.** La version 11 refuse de signer sans fournisseur
   cryptographique, et le refus est à l'exécution. `rust_crypto` retenu (Rust pur, cohérent avec
   `argon2`, pas de chaîne C pour le paquet auto-hébergé), `use_pem` écartée. Le gel n'a pas
   bougé : la version reste `=11.0.0`.
2. **Le hachage est livré en T020, pas en T025.** Les seeds en dépendent et l'ordonnancement ne
   voyait pas cette dépendance. T025 y ajoutera vérification, rehachage et condensat factice.
3. **Une sixième migration** — `0019_parametres_comptes.sql` — le plan en annonçait cinq. Les
   paramètres d'établissement vivent dans le schéma `etablissements`, pas dans `comptes` : ils
   méritaient leur propre fichier.

### Le piège de `cargo sqlx prepare` a livré sa cause

Consigné dans `CLAUDE.md`. **Deux passes obligatoires** : depuis `backend/` pour les tests, depuis
`backend/api/` pour les binaires, avec conservation des moissons hors de `.sqlx` entre les deux.
Aucun `clean`, `touch` ou `--all-targets` n'y change quoi que ce soit. La procédure exacte est
dans `CLAUDE.md`, section « Versions ».

---

## Le prompt à donner en session neuve

```text
/speckit-implement Reprends le cycle 003 (CPT) à la tâche T021 — les phases 1 et 2 sont
livrées et commitées, lis specs/003-comptes-roles-audit/REPRISE.md d'abord.

Implémente T021 à T064 dans l'ordre. Après chaque tâche : compile, teste, commite avec un
message conventionnel référençant la story (ex. "feat(comptes): CPT-02 l'union des
permissions, BTreeSet dans la signature du trait").

Trois choses à ne pas redécouvrir :
- `cargo sqlx prepare` exige DEUX passes, une depuis backend/ et une depuis backend/api/ —
  la procédure exacte est dans CLAUDE.md, section « Versions ». Une seule passe fait échouer
  le check hors ligne sur des requêtes que prepare vient pourtant d'annoncer avoir écrites.
- Le hachage Argon2 existe déjà (`authentification/argon2.rs`, livré avec T020) : T025 lui
  ajoute la vérification, le rehachage et le condensat factice, il ne le réécrit pas.
- La taxonomie d'audit est à 0/10 branchés. T039 et T041 doivent faire passer
  `changement_role` et `suppression` à « branché » dans docs/taxonomie-audit.md, DANS le
  même changement que le code qui les écrit — sinon backend/tests/audit_taxonomie.rs fait
  échouer le build, ce qui est exactement son rôle.

Ne saute jamais la régénération du client TypeScript. À la fin, déroule la checklist des
vingt points de la Definition of Done et liste ce qui resterait non conforme.
```

---

## Ce qui attend en T021, et les pièges connus

**T021 à T024 (US1)** — les trois tables jamais confondues. Le contrôle statique de T021 doit
lire `information_schema.columns` et échouer si une colonne de contrat apparaît sur `compte` ou
`personne`. Les tables existent déjà (migrations `0015` et `0018`).

**T030 est la tâche la plus lourde du cycle** : elle refond `contexte.rs` **et**
`isolation_tenant.rs` en un seul passage, sur 21 opérations existantes. Ne pas la commencer un
vendredi. La dérogation `CONTEXTE_PAR_EN_TETES` est levée par elle, et par elle seule.

**Deux provisoires nommés attendent leur levée**, et le code les nomme :

- `app/modules/etablissements/bascule-service.ts` — `PERMISSION_BASCULER`, « levé par CPT-02 » ;
- `app/pages/etablissement.vue` — « permissions — provisoire nommé, levé par CPT-02 ».

**Le test `parametres_catalogue.rs` fige le catalogue à six clés.** Toute clé ajoutée par un
cycle ultérieur passe par lui — c'est le moment où la question « est-ce vraiment un paramètre ? »
se pose.

**Les rôles opérationnels sont des sous-ensembles stricts de `gerant` à ce cycle.** Retirer
`caissier` à Adjoua ne retire donc aucune permission. La paire qui exerce réellement FR-018 est
`gerant` + `comptable`, dont `cpt.audit.consulter` est exclusive au second. C'est écrit dans
`0016_roles_permissions.sql` — ne pas « équilibrer » la table en inventant des permissions pour
les rôles pauvres.

---

## Vérifications d'entrée, à passer avant d'écrire la première ligne

```sh
docker compose -f infra/compose.yml up -d
cd backend && cargo test --workspace              # 145 tests attendus, 0 échec
cd .. && pnpm porte:p02 && pnpm porte:p05b && pnpm porte:p10
```

Si `cargo test` échoue sur `worker_redemarrage` ou `outbox_immuabilite` : un binaire de
développement tourne et consomme le grand livre. L'arrêter — le contrôle
`exiger_grand_livre_sans_consommateur_concurrent` le dira nommément.
