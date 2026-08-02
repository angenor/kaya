# Contrat interne `core/sync` — cycle 005 (SYN)

Ce que `app/core/sync` promet au reste de l'application. C'est la frontière la plus employée du
produit à partir de ce cycle : **tout écran qui écrit passe par elle**.

---

## 1. Ce qui existe déjà et ne change pas

| Export | Depuis | Rôle |
|---|---|---|
| `OperationClasseA<T>`, `marquerClasseA(charge, justification)` | cycle 001 | La marque **infalsifiable** — un symbole unique, pas un champ `classe: 'A'` que n'importe quel littéral satisferait |
| `TYPES_CLASSE_A`, `estTypeClasseA` | cycle 001 | La seconde barrière : un type non déclaré est refusé **même marqué** |
| `OperationRefusee` | cycle 001 | Le refus porte son motif |
| `viderFile(...)` | cycle 003 | **Point de sortie unique** — rafraîchir avant vider, jamais l'inverse |
| `brancherFile`, `ecrituresEnAttente`, `fileBranchee` | cycle 003 | La garde de « passer la main » |

**`marquerClasseA` reste le seul point d'entrée, et son paramètre `justification` n'est pas
décoratif** : il force à nommer la branche de l'arbre de décision du cadrage §11.2, et un appel
sans justification recevable se voit en revue. C'est le moment où la question « cette opération
est-elle vraiment de classe A ? » se pose — le seul.

---

## 2. Ce que ce cycle ajoute

### `FileLocale` — persistante

```ts
class FileLocale {
  static async ouvrir(adaptateur: PlatformAdapter): Promise<FileLocale>
  enfiler<T>(entree: EntreeFile<T>): void          // refuse hors classe A, au TYPE
  get enAttente(): number
  get enQuarantaine(): number
}
```

`ouvrir` est asynchrone parce que la clé de chiffrement vient du coffre système. **La file n'a
toujours aucun autre chemin de sortie que `viderFile`** — c'est ce qui porte l'ordre
rafraîchir-avant-vider, et non la discipline des appelants.

### `EntreeFile` — deux champs de plus

```ts
interface EntreeFile<T = unknown> {
  readonly id: string                    // UUID v7 client — existant
  readonly type: string                  // existant
  readonly horodatageClient: string      // indicatif — existant
  readonly charge: OperationClasseA<T>   // existant
  readonly contexte: ContexteEcriture    // NOUVEAU — { tenantId, etablissementId } à la SAISIE
  readonly tentatives: number            // NOUVEAU
}
```

> **`contexte` est figé à la saisie, jamais relu à l'envoi.** Aminata change d'établissement actif
> pendant une coupure : les écritures déjà enfilées **ne sont pas réattribuées**. Sans ce champ,
> elles partiraient sur le mauvais établissement au retour du réseau — une faute silencieuse, et
> impossible à démêler après coup.

> **Aucun jeton, toujours.** L'absence de champ est ce qui l'empêche : un jeton mis en file serait
> périmé au retour, et le ranger prolongerait la durée de vie d'un secret sur un terminal qu'on
> peut perdre.

### `EtatSynchronisation` — la source unique du témoin

```ts
function useEtatSynchronisation(): Readonly<Ref<{
  reseau: 'connecte' | 'degrade' | 'hors_ligne'
  enAttente: number
  enQuarantaine: number
}>>
```

**Trois états, jamais un pourcentage** — la règle du composant 10 est explicite : « un nombre
d'écritures et une heure, jamais une barre de progression ». Le passage à `hors_ligne` est
**instantané, sans transition**.

### `quarantaine()` — ce qui ne partira plus

```ts
function quarantaine(): readonly EntreeQuarantaine[]
function relancerDepuisQuarantaine(id: string): void   // geste explicite de l'utilisateur
```

**L'interface branche sa clé i18n sur `code`, jamais sur `message`** — la règle du lexique, qui
vaut ici comme ailleurs : le `message` nomme des tables et parle anglais technique.

---

## 3. Le contrat de refus — ce que l'écran doit faire, et quand

C'est la partie du contrat qui n'est pas du code, et c'est celle qu'on écrit mal.

| Situation | Ce que l'écran fait | Ce qu'il ne fait **jamais** |
|---|---|---|
| Opération **classe A**, hors ligne | Accepte, enfile, témoin à `n+1`, **aucun message d'erreur** | Bloquer, avertir, demander confirmation |
| Opération **classe B, C ou D**, hors ligne | Annonce l'indisponibilité **avant la saisie** | Griser en silence · échouer après coup · **mettre en file « au cas où »** |
| File non vide, geste « passer la main » | Refuse **immédiatement**, phrase du lexique | Purger · différer le refus · proposer de forcer |
| Refus définitif au rejeu | Quarantaine visible, motif en langue utilisateur | Rejet silencieux · réessai infini · blocage de la file derrière |

**Le vocabulaire est tenu par `docs/design/lexique.md`, et il est catégorique** : « idempotence,
rejeu, file d'attente — **n'apparaît jamais**. L'utilisateur voit *en attente d'envoi* et un
nombre. » Les clés i18n existent déjà (`reseau.connecte`, `reseau.degrade`, `reseau.hors_ligne`,
`reseau.en_attente`) — posées par un cycle antérieur, employées ici pour la première fois.

---

## 4. Les déclencheurs d'envoi, et celui qui doit suffire seul

| Déclencheur | Rôle |
|---|---|
| **Retour au premier plan** | Le déclencheur **par défaut**. Doit suffire seul, sur toutes les plateformes |
| Passage de l'état réseau à `connecte` | Opportuniste |
| Après une écriture réussie | La file profite d'un réseau qu'on vient de constater bon |
| Réessai à intervalle croissant plafonné | Après échec, jusqu'au prochain déclencheur naturel |

**Aucune minuterie de scrutation.** Réveiller la radio toutes les trente secondes coûterait la
batterie d'un service entier sur un Android d'entrée de gamme, pour un gain que le retour au
premier plan couvre déjà.

---

## 5. Portes touchées par ce contrat

| Porte | Ce que ce cycle y ajoute | Vérifié par |
|---|---|---|
| **P-13** | La marque de type **et** le balayage en direct des écrans, réseau coupé | `app/tests/file-classe-a.spec.ts` (compilation, `@ts-expect-error`) + nouveau cas e2e |
| **P-14** | Rejeu et désordre, désormais **engendrés par macro** pour toute entité de classe A | `backend/tests/commun/classes.rs` |
| **P-16** | Aucune chaîne en dur ; parité fr / en des clés nouvelles | `pnpm test:i18n` |
| **P-17** | Le témoin et `S1` n'emploient que des jetons | `pnpm lint:tokens` |
| **P-21** | La file ne charge rien d'un hôte externe ; WebCrypto est une API du moteur | `pnpm porte:p21` |
