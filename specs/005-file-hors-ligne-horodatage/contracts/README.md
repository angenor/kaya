# Contrats du cycle 005 (SYN)

Trois contrats, dont **un seul touche l'API HTTP** — et il ne la touche presque pas.

| Fichier | Frontière | Qui le consomme |
|---|---|---|
| [`api-http.md`](./api-http.md) | OpenAPI | Le client TypeScript généré (P-01) |
| [`platform-adapter.md`](./platform-adapter.md) | Application ↔ plateforme native | Les quatre adaptateurs, `desktop` / `android` / `ios` / `web` |
| [`sync-interne.md`](./sync-interne.md) | `core/sync` ↔ le reste de l'application | Tout écran qui écrit, et le témoin |

---

## Pourquoi l'API bouge si peu — et pourquoi c'est le bon signe

Le premier réflexe serait d'ajouter un endpoint de synchronisation par lots, ou un endpoint
d'horodatage serveur. **Ni l'un ni l'autre n'est nécessaire, et les ajouter serait une faute au
sens du principe X.**

- **Pas d'endpoint de lot.** Un lot introduit une sémantique d'échec partiel — quatre écritures
  acceptées, une refusée, que rend-on ? — que le rejeu idempotent rend inutile. L'envoi est
  unitaire ; chaque écriture emprunte l'endpoint que son module expose déjà.
- **Pas d'endpoint d'heure serveur.** Chaque réponse de création porte déjà l'horodatage
  d'autorité de la ligne créée. Le client y lit tout ce dont il a besoin pour savoir que **sa
  propre horloge** est fausse, qui est l'information utile à la personne qui tient le terminal.

Ce qui bouge est ailleurs : dans le **cycle de vie de l'application** (adaptateur de plateforme) et
dans **ce que `core/sync` promet à ses appelants**. Les deux contrats correspondants sont
inhabituels pour un cycle Spec Kit, et c'est exactement ce que SYN est.
