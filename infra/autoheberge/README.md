# Paquet auto-hébergé — mode B

**EMPLACEMENT SEUL. Rien n'est construit ici au cycle 001.**

`TRX-07` est une story **P1**, livrable après le cœur P0.

Le serveur, le nœud de site et le paquet auto-hébergé sont **le même binaire Actix avec trois
configurations** (constitution, § Pile technique imposée). Jamais trois produits. Ce répertoire ne
contiendra donc pas de code applicatif : un `compose.yml`, une configuration, une procédure de
mise à jour.

---

## Déjà livré par le cycle 001

| Élément | Où | Note |
|---|---|---|
| **Migrations automatiques et idempotentes au démarrage** | `backend/api/src/main.rs` (R-12) | Appliquées sous le rôle propriétaire, **avant** l'ouverture du port |
| Démarrage concurrent sûr | `backend/tests/migrations_concurrentes.rs` | Vérifié, pas supposé : deux conteneurs qui redémarrent ensemble ne se marchent pas dessus |
| Sonde `/health` | `backend/api/src/routes/sante.rs` | Publique, sans authentification — interrogeable par une supervision externe |
| Rôles amorçables à la main | `infra/postgres/init/00-kaya-owner.sql` | L'administrateur exécute trois ordres avant le premier démarrage |
| Lockfiles commités | `backend/Cargo.lock`, `pnpm-lock.yaml` | C'est ce qui rend la reconstruction identique à six mois d'écart — condition du support à distance |

C'est volontaire : rétrofiter les migrations au démarrage sur un socle déjà écrit coûte cher ; le
poser au premier cycle ne coûte rien.

---

## Ce qui reste dû à TRX-07

### La règle N / N-1 (cadrage §10.2)

**Versions N et N-1 supportées, pas plus.** C'est ce qui rend le support tenable pour un
développeur solo — mais rien ne l'implémente encore, et rien ne l'impose.

Il faut donc :

- une **télémétrie de version** — savoir quelle version tourne chez qui, sans quoi la règle est
  inapplicable (principe VIII : « télémétrie de version pour le parc auto-hébergé ») ;
- un **refus de démarrage** au-delà de N-2, avec un message qui indique la marche à suivre ;
- une **procédure de mise à jour** vérifiée sur une installation réelle, pas seulement rédigée.

### Le reste

| Élément | Note |
|---|---|
| `compose.yml` de production | Tags exacts du gel §4.2, jamais `latest` |
| Image `linux/amd64` | Construite **dans Docker**, jamais par copie d'un binaire local |
| Configuration d'exploitation | Secrets hors dépôt |
| Sauvegardes | `infra/backup/` est écrit ; l'exercice de restauration reste dû (voir son README §6) |

---

## Le piège à ne pas manquer

Un client auto-hébergé n'a **ni ingénieur, ni supervision, ni astreinte**. Tout ce qui suppose une
intervention manuelle régulière ne sera pas fait — et le découvrir en incident coûte la confiance.

C'est la raison d'être des migrations au démarrage : personne ne lancera une commande de migration
avant de relancer le service.
