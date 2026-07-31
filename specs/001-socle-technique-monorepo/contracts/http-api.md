# Contrat HTTP — cycle 001

**Source de vérité** : les annotations `#[utoipa::path]` dans le code Actix. **Ce document décrit
ce que le code doit produire ; il n'est jamais la référence** (constitution, principe I(a)).

Spec exposée sur `/api-docs/openapi.json`. Client TypeScript généré depuis cette spec en CI,
jamais écrit à la main. Un diff non commité fait échouer le build (porte P-01).

---

## 1. `GET /health` — sonde de santé

**Exigence** : FR-031 — le contrat doit documenter au minimum cet endpoint **dès ce cycle**.

| Aspect | Valeur |
|---|---|
| Authentification | Aucune |
| Contexte de tenant | **Aucun** — cet endpoint ne touche aucune table applicative |
| Réponse `200` | `EtatSante { statut, version, dependances }` |
| Réponse `503` | Même corps, `statut = "degrade"` |

```rust
#[utoipa::path(
    get,
    path = "/health",
    tag = "systeme",
    responses(
        (status = 200, description = "Service opérationnel",   body = EtatSante),
        (status = 503, description = "Service dégradé",        body = EtatSante),
    )
)]
```

```rust
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct EtatSante {
    pub statut: StatutSante,             // "operationnel" | "degrade"
    pub version: String,                 // version du binaire
    pub dependances: Vec<EtatDependance>,// base, cache, stockage objet
}
```

**Le point qui décide si la sonde sert à quelque chose** : elle vérifie les dépendances par une
requête réelle et **courte** (`SELECT 1` sur la base, `PING` sur le cache), pas l'état d'un pool
en mémoire. Un pool peut se croire sain pendant plusieurs minutes après la mort de la base — et
c'est exactement l'intervalle pendant lequel l'alerte des 2 minutes (FR-057) ne partirait pas.

**Ne renvoie jamais** : chaîne de connexion, nom d'hôte, version de PostgreSQL, trace d'erreur.
L'endpoint est public.

---

## 2. Module doré — notes d'établissement

Deux endpoints, une transition d'état, un événement. C'est le patron complet que les cycles
suivants recopient.

### 2.1 `GET /api/v1/etablissements/{etablissement_id}/notes`

```rust
#[utoipa::path(
    get,
    path = "/api/v1/etablissements/{etablissement_id}/notes",
    tag = "etablissements",
    params(
        ("etablissement_id" = Uuid, Path, description = "Identifiant de l'établissement"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Notes de l'établissement", body = PageNotes),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
    ),
    security(("bearer" = []))
)]
```

### 2.2 `POST /api/v1/etablissements/{etablissement_id}/notes`

```rust
#[utoipa::path(
    post,
    path = "/api/v1/etablissements/{etablissement_id}/notes",
    tag = "etablissements",
    params(("etablissement_id" = Uuid, Path)),
    request_body = CreerNoteRequete,
    responses(
        (status = 201, description = "Note créée",                    body = NoteEtablissement),
        (status = 200, description = "Note déjà créée (rejeu idempotent)", body = NoteEtablissement),
        (status = 400, description = "Requête invalide"),
        (status = 401, description = "Non authentifié"),
        (status = 403, description = "Permission absente"),
    ),
    security(("bearer" = []))
)]
```

```rust
#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct CreerNoteRequete {
    /// UUID v7 **généré par le client**. C'est lui qui rend le rejeu inoffensif.
    pub id: Uuid,
    pub texte: String,
    /// Indicatif — ordre d'affichage local. Jamais utilisé par une règle métier.
    pub horodatage_client: Option<OffsetDateTime>,
}
```

**Le `200` sur rejeu au lieu d'un `409` est un choix, pas un raccourci.** Un client hors ligne qui
rejoue sa file ne doit pas voir d'erreur pour une écriture que le serveur a déjà acceptée : la
constitution (principe VI) exige que le rejeu soit idempotent, et un `409` obligerait chaque
appelant à traiter un cas d'erreur qui n'en est pas un. Le corps renvoyé est la note telle qu'elle
est en base — **le serveur fait foi en conflit**.

---

## 3. Interface d'exploration du contrat

| Environnement | `/api-docs/openapi.json` | Swagger UI |
|---|---|---|
| Développement, test | Ouvert | Ouvert |
| **Production** | Ouvert | **Protégé** (FR-032) |

La protection est une **décision de configuration au démarrage**, pas un test de variable
d'environnement dispersé dans les handlers : le montage de la route Swagger UI est conditionnel.
Une route non montée ne peut pas fuir par oubli de garde.

---

## 4. Ce que le contrat n'expose pas à ce cycle

Aucun endpoint n'est exposé pour `evenement_outbox`, `mapping_comptable` ni `exercice_comptable`.

- L'**outbox** est un grand livre interne : l'exposer inviterait à le consulter par API et, à
  terme, à le filtrer, le paginer, puis le purger. Sa lecture se fera par les rapports de
  pilotage, jamais par un endpoint générique.
- Les **provisions comptables** sont des tables seulement (FR-047). Un endpoint, même en lecture,
  contredirait le principe X.
