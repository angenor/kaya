# Quickstart — valider le cycle 005 (SYN)

**But** : prouver, de bout en bout et sans lire le code, que la file survit à une coupure, que le
témoin dit vrai, qu'une opération de classe C refuse **avant** la saisie, et qu'une horloge fausse
est signalée sans bloquer personne.

Détails de conception : [plan.md](./plan.md) · [research.md](./research.md) ·
[data-model.md](./data-model.md) · [contracts/](./contracts/)

---

## 0. Prérequis

```sh
docker compose -f infra/compose.yml up -d          # Postgres 18.4, Redis 8.8.1, Garage 2.3.0
bash scripts/dev/preparer-base.sh                  # rôles, schémas, migrations
bash scripts/dev/preparer-stockage.sh
export KAYA_SEEDS_MOT_DE_PASSE=…                   # aucun mot de passe n'est écrit dans le dépôt
```

**⚠️ Ne jamais arrêter un processus par son nom de commande.** Cibler par port :
`lsof -ti:3000 | xargs kill`. `pkill -f "nuxt.mjs dev"` a déjà tué le serveur d'un autre projet de
ce poste.

---

## 1. Migrations et cache sqlx — la procédure qui ne ment pas

Deux migrations sont ajoutées, donc le cache de requêtes doit être régénéré. **La double passe est
obligatoire** : un `prepare` lancé depuis un seul répertoire perd les requêtes de l'autre.

```sh
cd backend
rm -rf /tmp/sqlx-a /tmp/sqlx-b && mkdir -p /tmp/sqlx-a /tmp/sqlx-b

cargo sqlx prepare --workspace -- --all-targets              # passe 1 — tests
git status --short .sqlx | grep '^??' | awk '{print $2}' | xargs -I{} cp {} /tmp/sqlx-a/
git checkout .sqlx

(cd api && cargo sqlx prepare --workspace -- --all-targets)  # passe 2 — binaires
git status --short .sqlx | grep '^??' | awk '{print $2}' | xargs -I{} cp {} /tmp/sqlx-b/
git checkout .sqlx

cp /tmp/sqlx-a/*.json /tmp/sqlx-b/*.json .sqlx/
```

**Puis les deux contrôles, dans cet ordre — le second seul ne suffit pas :**

```sh
git status --short backend/.sqlx    # AUCUNE suppression ; que des ajouts

grep -rl "sqlx::query" --include="*.rs" crates api tests | xargs touch
SQLX_OFFLINE=true DATABASE_URL= cargo check --workspace --all-targets --locked
```

> **Le `touch` n'est pas décoratif.** Sans lui, `cargo check` affiche `Finished` en une seconde
> **sans consulter `.sqlx`** — les macros ne sont pas réévaluées, et un cache périmé passerait au
> vert. C'est exactement ce qui est arrivé au cycle 004.

---

## 2. Scénario A — la file survit à une coupure *(User Story 1)*

```sh
pnpm --filter @kaya/app dev
```

1. Se connecter comme **Adjoua** (compte de démonstration, trois rôles).
2. Ouvrir `/notes`. Le **témoin** de la barre d'en-tête affiche « Connecté », zéro en attente.
3. **Couper le réseau** (mode hors ligne des outils de développement).
4. Enregistrer **quatre** notes internes.

**Attendu** — et chaque point est un échec s'il manque :

| Attendu | Ce que ça prouve |
|---|---|
| Les quatre écritures sont acceptées, **sans message d'erreur** | Classe A, différée légitimement (FR-002 de la story) |
| Le témoin passe à « Hors ligne » **instantanément**, sans transition | Composant 10 |
| Il affiche « **4 éléments en attente** », jamais un pourcentage | FR-025 |
| **Recharger la page** : les quatre sont toujours là | FR-012 — le cas fréquent, celui qu'on manque |
| Dans le stockage du navigateur, la charge est **illisible** | FR-013 |

5. Tenter « **Passer la main** ».

> Attendu : refus **immédiat** — « Des enregistrements ne sont pas encore partis. Attendez le
> retour du réseau avant de passer la main. » Le stockage **n'est pas purgé**.

6. **Rétablir le réseau**, puis repasser l'application au premier plan (changer d'onglet et
   revenir).

| Attendu | Ce que ça prouve |
|---|---|
| La session est rafraîchie **avant** tout envoi | FR-016 — ordre porté par le point de sortie unique |
| Les quatre notes arrivent, le témoin redescend à zéro | FR-014, FR-015 |
| Côté base : **quatre lignes, quatre événements outbox** | Aucun rejeu superflu |

---

## 3. Scénario B — le rejeu est inoffensif *(FR-018 à FR-018d)*

```sh
# Le même identifiant, trois fois.
for i in 1 2 3; do
  curl -s -o /dev/null -w "%{http_code}\n" -X POST \
    "$API/api/v1/etablissements/$ETB/notes" \
    -H "Authorization: Bearer $JETON" -H 'Content-Type: application/json' \
    -d '{"id":"0198c4a0-0000-7000-8000-0000000000aa","texte":"Groupe électrogène à 19 h 40."}'
done
```

**Attendu : `201`, puis `200`, puis `200`** — jamais `409`.

```sql
SELECT count(*) FROM etablissements.note_etablissement
 WHERE id = '0198c4a0-0000-7000-8000-0000000000aa';                       -- 1

SELECT count(*) FROM synchronisation.evenement_outbox
 WHERE payload->>'note_id' = '0198c4a0-0000-7000-8000-0000000000aa';      -- 1, PAS 3
```

> **Le second contrôle est le point du cycle.** Trois envois, **une ligne et un événement**.
> Émettre à chaque tentative ferait du grand livre le journal des tentatives réseau du terminal.

---

## 4. Scénario C — une action de classe C refuse avant la saisie *(User Story 2)*

Réseau coupé, ouvrir `/etablissement` et tenter d'**ajouter un service** (classe C).

> Attendu : « **Cette action nécessite internet.** » — annoncée **avant** la saisie, l'action
> indéclenchable. Jamais de grisé silencieux, jamais d'échec après coup, **jamais de mise en file**.

Le balayage automatisé de ce scénario sur **tous** les écrans d'écriture :

```sh
pnpm porte:p22                # parcours réel — chaque route s'ouvre
pnpm exec playwright test tests-e2e/hors-ligne.spec.ts    # FR-005b, réseau coupé
```

La seconde commande **rapporte le nombre d'opérations B/C/D couvertes** face au total du contrat.
Un écart fait échouer la porte en nommant l'opération manquante.

---

## 5. Scénario D — une horloge fausse est signalée, jamais bloquante *(User Story 3)*

```sh
curl -X POST "$API/api/v1/etablissements/$ETB/notes" \
  -H "Authorization: Bearer $JETON" -H 'Content-Type: application/json' \
  -d '{"id":"0198c4a0-0000-7000-8000-0000000000bb","texte":"Saisie hors ligne.",
       "horodatage_client":"2026-08-02T21:35:00Z"}'      # ~3 h dans le futur
```

| Attendu | Ce que ça prouve |
|---|---|
| **`201`** — l'écriture est **acceptée** | FR-036 : la dérive ne refuse jamais |
| `horodatage_client` conservé tel quel, `cree_le` = l'instant serveur | FR-031 |
| Une entrée `derive_horloge_constatee` au registre des actions | FR-035 |
| Rejouer dix fois : **une seule** entrée d'audit | Débrayage par épisode (R-04) |
| Même essai avec un horodatage **en retard** : signalé aussi | Valeur absolue, pas un dépassement |

Côté écran : l'utilisateur est averti que l'heure de son appareil est fausse, **sans que le mot
« dérive » ni aucune valeur technique n'apparaisse**.

---

## 6. Scénario E — l'outillage §0.7 s'instancie en une déclaration *(User Story 4)*

```rust
// Ce que doit coûter la couverture d'une entité de classe A à un cycle futur :
tester_classe_a!(note_etablissement, schema = "etablissements", creer = fabrique::note);
```

```sh
cargo test --test outillage_classes
```

> Attendu : les tests engendrés sont **nommés un par un** — six pour le désordre, un par ordre. Un
> test générique unique dirait « un des six ordres a échoué » sans dire lequel, et c'est ce qu'on
> lit en CI à 23 h.

**Le contrôle qui empêche l'oubli** : retirer l'instanciation d'une entité déclarée au registre
doit faire **échouer** le build en la nommant.

---

## 7. Les portes, une par une

```sh
# Backend
cargo test --workspace
cargo test --test classes_offline --test outillage_classes --test derive_horloge
cargo test --test horodatage_autorite        # P-23 — après amendement de constitution
cargo test --test provisions_sans_logique    # 6 provisions, reconciliation_orpheline NON écrivable

# Portes scriptées
pnpm porte:p01 && pnpm porte:p02 && pnpm porte:p04 && pnpm porte:p05b
pnpm porte:p10 && pnpm porte:p15 && pnpm porte:p19 && pnpm porte:p20
pnpm porte:p21 && pnpm porte:p21b
pnpm porte:p22 && pnpm porte:p22:negatif

# Application
pnpm lint && pnpm --filter @kaya/app test
pnpm --filter @kaya/app lint:tokens && pnpm --filter @kaya/app test:i18n
```

### Les deux bascules à vérifier — sinon un test passe pour la mauvaise raison

```sh
pnpm --filter @kaya/app test -- amorcage deconnexion
```

| Marqueur | Avant | Après |
|---|---|---|
| `brancherFile` à l'inventaire d'amorçage | « **dû** par SYN-01 » | « **branchée** » |
| Assertion de `deconnexion.spec.ts` | « aucune file n'est branchée » | « la file est branchée **et vide** » |

> Brancher la file **cassera** le test d'amorçage tant que l'entrée n'aura pas basculé — un test y
> échoue si une fonction déclarée due a un appelant. C'est le comportement attendu : c'est ce qui
> rend l'oubli impossible.

---

## 8. Ce que ce quickstart NE prouve pas

À lire avant de conclure que le cycle est validé.

- **La clôture au franc près (SC-009)** — la clôture journalière est de la tranche T3. Le test
  existe **installé à vide**, avec son assertion de non-régression : le cycle qui livrera la
  clôture le trouvera rouge, et c'est le but.
- **WKWebView** — le `webkit` de Playwright n'est pas WKWebView. Un vert dit « tourne sur un moteur
  WebKit », jamais « vérifié sur la cible ». Vaut particulièrement pour `crypto.subtle`, qui exige
  un contexte sécurisé (R-06). Le contrôle réel viendra avec la coquille Tauri.
- **La justesse des classes** — les portes vérifient qu'une classe a été **déclarée** et
  **exercée**, jamais qu'elle est **juste**. Aucune lecture du schéma ne peut retrouver
  qu'un encaissement est B en espèces et D en Mobile Money : c'est métier, et cela reste humain,
  revu mensuellement.
- **La garantie du stockage sur le web** — elle est `aucune`, et le type le dit. Le scénario A
  vérifie que la charge est illisible **sans la clé** ; sur le web, la clé est accessible à tout
  script de même origine, et le produit ne prétend pas le contraire.
