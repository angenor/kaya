# Contrat HTTP — cycle 005 (SYN)

**Le contrat OpenAPI est généré par utoipa depuis le code, jamais écrit à la main** (principe I·a).
Ce document décrit ce que le code doit produire ; le fichier qui fait foi est
`backend/api/openapi.json`, et le client TypeScript en est dérivé en CI (porte P-01).

---

## Aucune opération nouvelle

**Zéro endpoint créé, zéro endpoint modifié.** Le total d'opérations servies est inchangé, et
`p08_le_nombre_d_operations_servies_correspond_a_ce_qui_est_annonce` de
`backend/tests/couverture_portes.rs` garde son décompte.

C'est une décision, pas une omission — voir [`README.md`](./README.md).

### Ce que le premier passager de la file emploie, et qui existe déjà

| `operationId` | Verbe et chemin | Rôle dans ce cycle |
|---|---|---|
| `notes_creer` | `POST /api/v1/etablissements/{etablissement_id}/notes` | L'écriture de classe A que la file transporte |
| `notes_lister` | `GET  /api/v1/etablissements/{etablissement_id}/notes` | La lecture de l'écran, paginée |

`CreerNoteRequete` porte **déjà** les trois champs dont la file a besoin, et c'est le module doré
qui les y a mis :

```rust
pub struct CreerNoteRequete {
    /// UUID v7 **généré par le client**.
    pub id: Uuid,
    pub texte: String,
    /// Indicatif : ordre d'affichage local. **Jamais utilisé par une règle métier.**
    pub horodatage_client: Option<OffsetDateTime>,
}
```

**Rien à ajouter.** Un cycle qui aurait « oublié » ces champs devrait ici modifier une entité
existante, régénérer le contrat et le client, et rouvrir P-01, P-01b et P-02. Le module doré les a
posés au cycle 001 précisément pour que ce cycle-ci n'ait rien à rouvrir.

---

## Les réponses, et la seule qui compte pour la file

| Code | Sens | Ce que la file en fait |
|---|---|---|
| `201 Created` | La note n'existait pas | Retire l'entrée de la file |
| `200 OK` | **Elle existait déjà** — corps = la ligne telle qu'elle est en base | Retire l'entrée de la file. **Chemin normal d'un rejeu, pas une erreur** |
| `401` | Session expirée | **N'atteint jamais la file** : le point de sortie unique rafraîchit d'abord (FR-016) |
| `408`, `429`, `5xx` | Le serveur dit « plus tard » | Réessai à intervalle croissant |
| `400`, `403`, `404`, `409`, `422` | Refus **définitif** | Quarantaine, avec le `code` en clé i18n |

> **`200` est le cas qu'on écrirait mal.** Une file qui traiterait « déjà présente » comme un
> conflit remettrait l'écriture en tête et boucllerait indéfiniment. Le patron rend `200` avec la
> ligne en base **pour que ce cas soit le chemin normal** — jamais `409`, jamais une erreur.

---

## Ce que le serveur fait en plus, sans que le contrat change

À l'ingestion de toute écriture portant un `horodatage_client`, le service constate la dérive
(R-04) et, au-delà du seuil paramétré :

- écrit une entrée `derive_horloge_constatee` au registre des actions, **débrayée par épisode** ;
- **accepte l'écriture** — la dérive n'est jamais un motif de refus (FR-036).

Aucun champ de réponse nouveau : le client déduit la dérive de l'horodatage d'autorité que la
réponse porte déjà, comparé à sa propre horloge.

---

## Portes touchées par ce contrat

| Porte | Effet de ce cycle | Vérifié par |
|---|---|---|
| **P-01** | Le client est régénéré ; **aucun diff attendu**, le contrat ne changeant pas | `pnpm porte:p01` |
| **P-01b** | Aucun `operationId` ajouté ; l'unicité reste acquise | `p01b_les_operation_id_du_contrat_sont_tous_presents_et_distincts` |
| **P-08** | Le décompte d'opérations servies est **inchangé** — et c'est le contrôle qui le prouve | `p08_le_nombre_d_operations_servies_correspond_a_ce_qui_est_annonce` |

**Un contrat inchangé n'est pas un contrat non vérifié.** P-01 doit être exécutée et rendre un
diff vide ; un contrat qui aurait dérivé sans qu'on l'ait voulu se verrait exactement là.
