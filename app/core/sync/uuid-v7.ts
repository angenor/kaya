/**
 * **UUID v7 généré côté client** — la brique du principe VI.
 *
 * > Toute écriture porte un UUID v7 généré côté client ; le serveur déduplique ; le rejeu est
 * > idempotent ; le serveur fait foi en conflit.
 *
 * C'est cet identifiant qui rend le rejeu inoffensif : trois envois de la même écriture entrent en
 * conflit de clé primaire et produisent **un** enregistrement (`docs/module-dore.md`, couche 1).
 * Une clé générée par la base en produirait trois, et le terminal qui vide sa file après une
 * coupure créerait des doublons silencieux — découverts trois mois plus tard, en clôture.
 *
 * # Pourquoi v7 et pas `crypto.randomUUID()`
 *
 * `crypto.randomUUID()` produit un **v4**, entièrement aléatoire. Le v7 préfixe 48 bits
 * d'horodatage : les identifiants sont **ordonnés dans le temps**, ce dont le produit dépend en
 * deux endroits déjà écrits — l'ordre secondaire `ORDER BY cree_le DESC, id DESC` du repository,
 * qui départage deux lignes créées dans la même transaction, et la localité d'insertion des index
 * B-tree. Un v4 y ferait sauter la pagination et fragmenterait l'index.
 *
 * # Pourquoi pas une dépendance
 *
 * Le format est **quinze lignes** (RFC 9562 §5.7) et n'évoluera plus. Une dépendance de plus, c'est
 * une ligne de plus au gel, une revue mensuelle de plus, et un paquet de plus dans un bundle qui
 * part sur un Android d'entrée de gamme.
 *
 * # L'horloge employée ici est LOCALE, et c'est correct
 *
 * L'horodatage d'autorité serveur (principe IV) régit les **durées, les taxes et les clôtures**.
 * Celui-ci ne régit rien : c'est un identifiant. Un terminal mal réglé produit un identifiant à
 * l'ordre légèrement décalé, jamais un montant faux. La distinction est celle que le module doré
 * pose entre `horodatage_client` (indicatif) et `cree_le` (autorité).
 */

/**
 * Un UUID version 7, en minuscules et avec ses tirets.
 *
 * Disposition RFC 9562 §5.7 : 48 bits de millisecondes Unix, 4 bits de version (`7`), 12 bits
 * aléatoires, 2 bits de variante (`10`), 62 bits aléatoires.
 */
export function uuidV7(): string {
  const octets = new Uint8Array(16)
  crypto.getRandomValues(octets)

  // 48 bits d'horodatage, gros-boutiste. `Date.now()` dépasse 2^32 : le décalage se fait en
  // arithmétique flottante, pas en opérateurs binaires — `>>> 32` sur un nombre JavaScript
  // retomberait sur 32 bits et écraserait les deux octets de poids fort.
  const millisecondes = Date.now()
  octets[0] = Math.floor(millisecondes / 2 ** 40) & 0xff
  octets[1] = Math.floor(millisecondes / 2 ** 32) & 0xff
  octets[2] = Math.floor(millisecondes / 2 ** 24) & 0xff
  octets[3] = Math.floor(millisecondes / 2 ** 16) & 0xff
  octets[4] = Math.floor(millisecondes / 2 ** 8) & 0xff
  octets[5] = millisecondes & 0xff

  // Version 7 sur les quatre bits de poids fort de l'octet 6 ; variante RFC 4122 (`10`) sur les
  // deux bits de poids fort de l'octet 8. Le reste est déjà aléatoire.
  octets[6] = (octets[6] & 0x0f) | 0x70
  octets[8] = (octets[8] & 0x3f) | 0x80

  const hexa = [...octets].map(o => o.toString(16).padStart(2, '0')).join('')
  return [
    hexa.slice(0, 8),
    hexa.slice(8, 12),
    hexa.slice(12, 16),
    hexa.slice(16, 20),
    hexa.slice(20, 32),
  ].join('-')
}
