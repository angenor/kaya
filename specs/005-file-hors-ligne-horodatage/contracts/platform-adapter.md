# Contrat `PlatformAdapter` — cycle 005 (SYN)

**Aucune invocation directe de `window.__TAURI__` dans un composant** (principe VII, porte P-15).
Le cycle de vie de l'application est une capacité de plateforme au même titre que l'impression et
la géolocalisation : ce que le produit ajoute ici passe par l'adaptateur, et par lui seul.

---

## 1. Capacité nouvelle — `surRetourPremierPlan`

```ts
export interface PlatformAdapter {
  // … capacités existantes inchangées …

  /**
   * S'abonne au retour de l'application au premier plan.
   * Rend la fonction de désabonnement — jamais `void` : un écouteur qu'on ne peut pas retirer
   * fait fuir la mémoire à chaque navigation, sur des terminaux qui n'en ont pas.
   */
  surRetourPremierPlan(rappel: () => void): () => void
}
```

### Ce que chaque plateforme branche dessus

| Adaptateur | Signal | Note |
|---|---|---|
| `web` | `visibilitychange` (vers `visible`) **et** `focus` | Les deux, parce qu'un changement d'onglet et un retour de fenêtre ne produisent pas le même événement |
| `desktop` | Événement de focus de fenêtre Tauri | Plus fin que le signal du navigateur — c'est la raison d'être de l'abstraction |
| `android` | Reprise d'activité | `WorkManager` **n'est pas ici** : MOB-06, optimisation |
| `ios` | Retour au premier plan | `BGTaskScheduler` **n'est pas ici** : iOS n'a pas de synchronisation en arrière-plan, et le produit doit être complet sans elle |

> **La règle qui décide de ce contrat** : la file est **conçue pour se vider au retour au premier
> plan par défaut, sur toutes les plateformes**. Tout le reste est optimisation. Un adaptateur qui
> ne saurait pas signaler le retour au premier plan rendrait le produit inutilisable sur sa cible —
> il n'y a donc pas de `ResultatCapacite` ici, pas de `CapaciteIndisponible` : c'est un socle, pas
> une commodité.

---

## 2. `etatReseau()` — même signature, source enrichie

La signature ne change pas. Ce qui change est **ce qui l'alimente**.

```text
plateforme dit « hors ligne »                        → 'hors_ligne'
plateforme dit « en ligne » ET dernier appel KO      → 'degrade'
plateforme dit « en ligne » ET dernier appel > seuil → 'degrade'
sinon                                                → 'connecte'
```

`app/core/platform/reseau.ts` porte déjà, en commentaire de tête, la ligne que ce cycle honore :

> « D'où le troisième état, `degrade`, que **le cycle SYN alimentera depuis les échecs réels de
> requête** — il n'est produit par personne aujourd'hui, et c'est écrit plutôt que supposé. »

**Le seuil est un paramètre d'établissement** (`sync.latence_degradee_seuil_ms`, défaut 3000), pas
une constante. Sans seuil nommé, l'état « dégradé » ne serait pas testable et une porte ne pourrait
pas le distinguer de « connecté ».

### Ce que `connecte` signifie, et ne signifie pas

`connecte` veut dire « rien ne dit que c'est coupé », **jamais** « ça marche ». Une opération de
classe C peut donc encore échouer après coup, et son message d'erreur doit rester lisible. Ce
n'est pas parce que la garde hors-ligne existe qu'elle dispense du traitement d'erreur — la
remarque figure déjà dans `reseau.ts` et reste vraie après ce cycle.

---

## 3. `stockageSecurise` — usage nouveau, contrat inchangé

Aucune méthode ajoutée. Ce cycle **emploie** `lire` / `ecrire` / `purger` pour une clé, et une
seule : celle qui chiffre la file.

```text
clé de chiffrement de la file   →  stockageSecurise      (coffre système, ou 'aucune' sur web)
cryptogramme de la file         →  stockage persistant ordinaire de la plateforme
```

**Le coffre système n'est pas un magasin** (R-06) : Keystore et Keychain servent des secrets courts
et peu nombreux. Y ranger une file réécrite à chaque saisie est un usage qu'ils tiennent mal, et
qui échouerait d'abord sur l'Android d'entrée de gamme d'Aminata — la cible.

### La garantie du web est déclarée, pas maquillée

`NiveauGarantieStockage` porte le niveau **dans le type** précisément pour cela :

| Adaptateur | `garantie` | Conséquence assumée |
|---|---|---|
| `desktop`, `android`, `ios` | `coffre_systeme` | La clé ne sort pas de l'appareil |
| `web` | **`aucune`** | Un script de même origine peut lire la clé. La contrepartie est portée ailleurs : purge à la déconnexion, rotation des jetons, coupure depuis « Appareils connectés » |

**Le produit ne prétendra pas que le web est sûr.** L'appelant lit la garantie avant de décider —
c'est tout l'objet d'avoir mis le niveau dans le type plutôt que dans un commentaire.

---

## 4. `purger()` — l'ordre du geste « passer la main » est confirmé, pas modifié

Le layout pose déjà la garde : `ecrituresEnAttente() > 0` refuse **immédiatement**, avant toute
purge. Ce cycle ne change pas l'ordre ; il rend la garde **effective**, la file étant enfin
branchée.

Deux marqueurs posés au cycle 003 basculent dans le même changement, et le manquer ferait passer
un test **pour la mauvaise raison** :

| Marqueur | État actuel | Après ce cycle |
|---|---|---|
| `brancherFile` à l'inventaire d'amorçage | « **dû** par SYN-01 » | « **branchée** » |
| Assertion de `deconnexion.spec.ts` | « aucune file n'est branchée » | « la file est branchée **et vide** » |

Le test d'amorçage échoue si une fonction déclarée due a un appelant : brancher la file **cassera**
ce test tant que l'entrée n'aura pas basculé. C'est le comportement attendu, et c'est ce qui rend
l'oubli impossible.

---

## 5. Portes touchées par ce contrat

| Porte | Ce que ce cycle y ajoute | Vérifié par |
|---|---|---|
| **P-15** | Une capacité de plus dans l'adaptateur ; **aucun `@tauri-apps/api` hors de lui** | `pnpm porte:p15` — décompte des fichiers analysés par arbre |
| **P-22** | Le nouvel écran et le témoin sont atteints en direct et par navigation, deux thèmes, deux moteurs | `pnpm porte:p22` |
| **Exigence 6** *(couverture des portes)* | `surRetourPremierPlan` et `brancherFile` sont des **fonctions d'amorçage** : deux preuves dues chacune — un test qui les exerce, et un test qui vérifie qu'elles sont appelées dans le parcours réel | `app/tests/amorcage.spec.ts` |
