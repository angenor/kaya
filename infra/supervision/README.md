# Supervision externe — Kaya

**FR-057 : alerte au-delà de 2 minutes d'indisponibilité.**

## La seule règle qui compte

**La supervision est hébergée hors du serveur surveillé.** Un serveur mort n'envoie pas d'alerte
disant qu'il est mort — une sonde installée sur la machine qu'elle surveille ne prouve rien, elle
donne seulement l'illusion d'une surveillance.

Le support se fait depuis Abidjan, à **220 km** du pilote d'Abengourou. Entre le moment où le
service tombe et celui où le gérant appelle, il peut s'écouler une matinée entière de service —
c'est-à-dire une matinée de retour au papier.

## Ce qui est surveillé

| Cible | Attendu | Seuil d'alerte |
|---|---|---|
| `GET https://<hôte>/health` | `200`, `statut = "operationnel"` | **2 minutes** d'échec continu |
| `GET https://<hôte>/health` | `dependances[].statut` | Une dépendance `degrade` pendant 5 min |
| Certificat TLS | Valide | 14 jours avant expiration |

Le `503` renvoyé par la sonde en cas de dépendance muette est **volontaire** : la supervision lit
le code de statut, pas le corps. Voir `backend/api/src/routes/sante.rs`.

## Réglage

Deux minutes est un seuil, pas une cadence : interroger toutes les 30 secondes et alerter après
**quatre échecs consécutifs** évite l'alerte sur un incident réseau d'une seconde tout en tenant
le seuil.

Intervalle 30 s · seuil 4 échecs · délai de requête 5 s (`/health` répond en moins de 2 s par
construction — voir `DELAI_SONDE`).

## État au 2026-07-31 — cycle 001

**Non provisionnée.** Ce document arrête ce qui doit être surveillé, avec quels seuils et
pourquoi ; le choix du service de supervision et son abonnement relèvent de l'exploitation, pas du
code, et supposent un **nom de domaine et un serveur de production** qui n'existent pas encore.

Aucune dépendance de code n'en découle : la sonde `/health` est livrée, publique et sans
authentification — précisément pour qu'un service externe puisse l'interroger sans rien savoir du
produit.

**Ce qui reste dû** : provisionner le service au premier déploiement, et consigner ici son nom,
son URL de tableau de bord et le canal d'alerte.
