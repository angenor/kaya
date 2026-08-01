# Conformité — registre des traitements ARTCI

**EMPLACEMENT SEUL pour le registre ARTCI. Rien n'en est construit au cycle 001.**

> **Ce répertoire porte désormais un document livré** : `licences-tierces.md`, l'inventaire de ce
> que Kaya **redistribue** dans son binaire — trois polices, deux licences, une modification
> déclarée. Il est ici parce que c'est une obligation de conformité comme les autres, et parce
> qu'un inventaire de licences rangé ailleurs ne se retrouve pas le jour où on le cherche. La porte
> **P-21b**, contrôle 5, en vérifie la moitié mécanisable.

`TRX-06` est une story **P1**, livrable après le cœur P0 (`docs/user-stories-v1.md` §0.5). Ce
répertoire existe pour que sa place soit prise et que son périmètre soit écrit — pas pour anticiper
son contenu.

---

## Ce que TRX-06 devra livrer

| Élément | Exigence |
|---|---|
| **Registre des traitements** | Inventaire des traitements de données personnelles, tenu à jour |
| **Export des données d'une personne** | Sur demande, dans un format lisible |
| **Suppression des données d'une personne** | Avec sa conséquence sur les documents fiscaux — voir ci-dessous |
| **Rétention paramétrable** | Par type de donnée, en configuration d'établissement |
| **Consentement tracé** | Qui, quand, pour quoi |

---

## Deux points à trancher avant d'écrire une ligne

### 1. Le transfert transfrontalier est déjà engagé

**B-01 est tranchée de fait par le choix Contabo** : le serveur est en Europe, et le pilote est en
Côte d'Ivoire. Les pièces d'identité des clients de l'hôtel franchissent donc une frontière dès le
premier check-in.

Ce n'est pas une question ouverte que TRX-06 poserait : c'est un état de fait que TRX-06 devra
**documenter et encadrer**, ou faire changer. Le noter ici évite qu'on le découvre en rédigeant le
registre.

### 2. Suppression contre conservation fiscale — les deux obligations se contredisent

Le droit à l'effacement d'une personne se heurte à la **conservation des documents fiscaux pendant
dix ans**, et au **grand livre d'événements immuable à rétention illimitée** (TRX-02) dont la
charge utile est dénormalisée — elle porte donc des noms de clients en clair.

Trois voies, à arbitrer avec un juriste, pas en écrivant du code :

1. **Anonymisation par événement de compensation** — un nouvel événement neutralise les données
   personnelles des précédents, sans rien supprimer. Cohérent avec l'immuabilité, mais laisse les
   données originales en base.
2. **Pseudonymisation à l'écriture** — la charge utile porte un pseudonyme, la correspondance vit
   dans une table effaçable. Contredit la reconstitution autonome de TRX-02 : le lecteur du grand
   livre ne pourrait plus dire *qui*.
3. **Exception fiscale documentée** — la conservation légale prime, l'effacement porte sur le reste.

**Aucune n'est gratuite, et le choix rétroagit sur le format de charge utile.** C'est pourquoi
`version_schema` existe (R-06) : changer de format sera possible, avec un décodeur par génération.

---

## Voir aussi

- `docs/cadrage-v1.md` — annexe B, décision **B-01**
- `.specify/memory/constitution.md` — principe IX (sécurité), principe X (périmètre)
- `specs/001-socle-technique-monorepo/data-model.md` §4.2 — format de la charge utile
