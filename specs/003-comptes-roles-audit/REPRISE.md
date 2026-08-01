# Reprise du cycle 003 (CPT) — état au 2026-08-01, seconde session

*T001 à T032 livrées. Reste **T033 à T064** — la couche front de US2, puis les phases 5 à 10.*

---

## Où en est le cycle

**32 tâches sur 64.** Phases 1, 2, 3 complètes ; **phase 4 livrée pour sa moitié backend**
(T025 à T032), sa moitié front (T033 à T036) reste à faire.

**197 tests backend**, 0 échec. **214 tests front**, 0 échec (inchangés — le front n'a pas encore
été touché par ce cycle). Portes vérifiées localement et vertes : P-01, P-02, P-04, P-05b, P-10,
P-15, P-19, P-20, P-21, P-21b.

### Ce que la seconde session a livré

| Tâche | Livré |
|---|---|
| T021 | `personne_compte_employe.rs` — trois figures + deux contrôles outillés, test négatif exercé |
| T022–T023 | Sous-module `personne` (3 couches) et ses **trois** points d'entrée. Aucune liste, par décision |
| T024 | Provisions étendues à `employe` et `appareil_enrole` — **aucun privilège**, pas même `SELECT` |
| T025 | Argon2 : vérification, rehachage, **condensat factice** en `OnceLock` + préchauffage |
| T026 | `politique.rs` — 8 caractères, aucune règle de composition, refus des compromis |
| T027 | `session/` — deux jetons, **trois familles de clés Redis**, durées lues du catalogue |
| T028 | Service d'authentification + `authentification_indiscernable.rs` (mesure de durées) |
| T029 | `limite.rs` — fenêtre glissante sur **deux** clés, refus indiscernable |
| T030 | **Dérogation `CONTEXTE_PAR_EN_TETES` levée.** `contexte.rs` et `isolation_tenant.rs` refondus |
| T031 | `routes/session.rs` — les six opérations, dont les deux seules publiques |
| T032 | `session_revocation.rs` — quatre propriétés, dont la famille qui tombe entière |
| — | Client TypeScript régénéré (**33 opérations**), P-01 verte |

### Quatre écarts au plan, assumés et consignés

1. **Une septième migration : `0020_resolution_identifiant.sql`.** Ni le plan ni le modèle de
   données n'avaient vu que la connexion doit trouver un compte **sans connaître le tenant**,
   alors que `FORCE ROW LEVEL SECURITY` l'interdit. Le fichier écrit les **quatre** solutions
   envisagées ; celle qu'on écrit d'abord — `SECURITY DEFINER` détenue par `kaya_owner` — **ne
   marche pas**, parce que `FORCE` s'applique au propriétaire : la fonction ne rend rien et ne
   lève rien. La retenue est un rôle `kaya_auth` **NOLOGIN**, une politique `FOR SELECT` sur la
   seule table `comptes.compte`, et une fonction dont le périmètre **est** la signature.
2. **Deux sous-modules non prévus comme tâches : `compte/` et `roles/`.** Le service
   d'authentification en avait besoin (résolution d'identifiant, union des permissions). Leurs
   `service.rs` restent à écrire — T039 pour les rôles, T041 pour les comptes.
3. **`suppression` est passée à « branché » en T028, pas en T041.** Le document annonçait la
   désactivation de compte ; c'est la **révocation de session** qui a branché le type la première.
   Le harnais l'a signalé au moment exact où le premier chemin d'écriture est apparu.
4. **La révocation croisée n'est pas livrée.** `session_revoquer` ne coupe que les sessions du
   compte appelant : il n'existe pas d'annuaire des sessions par tenant. La garde de permission
   existe, la fonctionnalité viendra avec CPT-05 (tranche T4). Écrit dans le handler.

### Trois pièges rencontrés, et leurs symptômes trompeurs

- **La partie aléatoire d'un UUID v7 est à la FIN.** Ses 48 premiers bits sont l'horodatage : deux
  UUID de la même seconde partagent leurs douze premiers caractères hexadécimaux. Un identifiant
  de test taillé dans le **préfixe** collisionne entre tests parallèles, le compte résolu est
  celui d'un autre tenant, et le test échoue sur un message qui accuse le handler.
- **`current_setting('app.current_tenant', true)` rend la chaîne vide, pas `NULL`**, dès qu'une
  transaction antérieure de la même connexion l'a posé. La politique la convertit en `uuid` et
  échoue en `22P02`. Poser le tenant avant tout décompte, même en lecture.
- **PostgreSQL 16+ : `CREATEROLE` obtient `ADMIN OPTION` mais `SET FALSE`.** `ALTER FUNCTION …
  OWNER TO` échoue sur « must be able to SET ROLE » alors qu'on vient de créer le rôle. Et un
  `GRANT` au propriétaire **se fond dans son entrée d'ACL, que le changement de propriétaire
  remplace** : il doit venir après, sous `SET LOCAL ROLE`.

---

## Le prompt à donner en session neuve

```text
/speckit-implement Reprends le cycle 003 (CPT) à la tâche T033 — les tâches T001 à T032 sont
livrées et commitées, lis specs/003-comptes-roles-audit/REPRISE.md d'abord.

Implémente T033 à T064 dans l'ordre. Après chaque tâche : compile, teste, commite avec un
message conventionnel référençant la story.

Quatre choses à ne pas redécouvrir :
- `cargo sqlx prepare` exige DEUX passes, une depuis backend/ et une depuis backend/api/ —
  procédure exacte dans CLAUDE.md, section « Versions ». Ce n'est TOUJOURS PAS fait pour ce
  cycle : `.sqlx` est en retard de toutes les requêtes de CPT-00 et CPT-01, et le check hors
  ligne échouera tant que T061 n'est pas passée.
- Le backend de US2 est complet et testé : `ServiceAuthentification`, les six opérations de
  session, la liste de révocation. T033 branche le FRONT dessus, il ne réécrit rien du
  serveur. Le client TypeScript est à jour (33 opérations).
- La dérogation CONTEXTE_PAR_EN_TETES est levée côté serveur, et le front la pose ENCORE :
  `app/modules/etablissements/donnees.ts` et `bascule-service.ts` envoient `x-kaya-tenant` et
  `x-kaya-compte`, que l'API n'accepte plus. L'écran G1 est donc cassé jusqu'à T033 — c'est
  attendu, l'ordonnancement le prévoit, et T033 doit le réparer en même temps qu'il livre R0.
- La taxonomie d'audit est à 1/10 branchés (`suppression`, branchée par la révocation de
  session). T039 doit faire passer `changement_role` à « branché » dans
  docs/taxonomie-audit.md, DANS le même changement que le code qui l'écrit — sinon
  backend/tests/audit_taxonomie.rs fait échouer le build, ce qui est exactement son rôle.

Ne saute jamais la régénération du client TypeScript. À la fin, déroule la checklist des
vingt points de la Definition of Done et liste ce qui resterait non conforme.
```

---

## Ce qui attend en T033, et les pièges connus

**T033 doit réparer l'écran G1 en même temps qu'il livre la connexion.** Le serveur n'accepte plus
les deux en-têtes ; `app/modules/etablissements/donnees.ts` (ligne 62) et `bascule-service.ts`
(ligne 157) les posent encore. Remplacer par `Authorization: Bearer`, et **lever le provisoire
nommé** de `donnees.ts` (« contexte d'appel — provisoire, levé par CPT-01 »).

**Le stockage sécurisé du web n'existe pas encore.** `app/core/platform/index.ts` expose
`stockageSecuriseAbsent`, qui refuse tout — c'est ce que `web.ts` emploie. T033 exige « stockage
adapté sur web » : il faut donc écrire cette implémentation, et **dire ce qu'elle ne garantit
pas**. `localStorage` n'est pas un stockage sécurisé ; le faire passer pour tel donnerait à
l'appelant une garantie fausse.

**T034 est la tâche dont le défaut ne se voit pas en développement.** La file de classe A doit
**rafraîchir avant de vider**, jamais l'inverse, et le test doit échouer si l'ordre s'inverse *y
compris quand les deux réussissent*. En développement la coupure dure trente secondes et le défaut
ne se manifeste pas ; il perd un service entier à Abengourou.

**T035 (`R0`) est codable** : la ligne a été ajoutée à `docs/design/derivation.md` par T001.
`G3` et `G4` y figuraient déjà.

**Les rôles opérationnels restent des sous-ensembles stricts de `gerant`.** Retirer `caissier` à
Adjoua ne retire aucune permission. La paire qui exerce réellement FR-018 est `gerant` +
`comptable`, dont `cpt.audit.consulter` est exclusive au second.

**`backend/tests/couverture_portes.rs` compte par lot.** P-08 en est à **33 opérations sur 40** :
1 sonde + 2 notes + 21 (cycle 002) + 3 personnes + 6 session. Restent 7 — les sept opérations de
comptes et rôles de T041, plus les deux référentiels et le journal d'audit… soit 9. **L'écart de
deux est à trancher en T060** : le contrat annonce 19 opérations pour ce cycle et en a livré 9 ;
les 10 restantes (7 comptes/rôles + 2 référentiels + 1 audit) portent le total à 43, pas 40. Le
contrat compte `compte_lire` et `compte_lister` comme deux opérations sur des chemins différents,
et `personne_lire`/`personne_modifier` partagent un chemin. **Recompter au recollement plutôt que
d'ajuster un chiffre.**

---

## Vérifications d'entrée, à passer avant d'écrire la première ligne

```sh
docker compose -f infra/compose.yml up -d
cd backend && cargo test --workspace              # 197 tests attendus, 0 échec
cd ../app && pnpm test                            # 214 tests attendus, 0 échec
cd .. && pnpm porte:p01 && pnpm porte:p04 && pnpm porte:p05b && pnpm porte:p10
```

Si `cargo test` échoue sur `worker_redemarrage` ou `outbox_immuabilite` : un binaire de
développement tourne et consomme le grand livre. L'arrêter — le contrôle
`exiger_grand_livre_sans_consommateur_concurrent` le dira nommément.

**`SQLX_OFFLINE=true cargo check` échoue, et c'est attendu** : le cache `.sqlx` n'a pas été
régénéré depuis le cycle 002. C'est T061.
