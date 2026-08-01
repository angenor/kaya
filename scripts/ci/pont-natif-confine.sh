#!/usr/bin/env bash
#
# Porte P-15 — aucune invocation de `window.__TAURI__` hors de `PlatformAdapter`.
#
#     pnpm porte:p15
#
# Deux moitiés, et la seconde est celle qui manquait.
#
#   1. **ESLint s'exécute** sur `app/` et `web/`, avec la configuration racine `eslint.config.js`.
#   2. **La couverture est comptée**, arbre par arbre. Une porte dont la cible est vide passe
#      toujours : c'est l'exigence 4 de « Couverture des portes », et le cycle 002 l'a déjà payée
#      deux fois — ESLint ne parsait aucun `.vue` typé, et `web/` n'était couvert par aucune
#      configuration du tout.
#
# ─────────────────────────────────────────────────────────────────────────────────────────────────
#  POURQUOI `web/` COMPTE PLUS QUE `app/` ICI
# ─────────────────────────────────────────────────────────────────────────────────────────────────
#
# `web/qr` est ouverte par un client sur son propre téléphone, sans compte et sans application
# installée. `web/console` est la console éditeur, servie par un navigateur. **Ni l'une ni l'autre
# ne tourne dans une coquille Tauri.** Un import de `@tauri-apps/api` n'y casse pas « plus tard, sur
# une autre plateforme » comme dans `app/` : il casse tout de suite, pour tous les visiteurs.
#
# La porte y était donc plus critique qu'ailleurs — et c'est exactement là qu'elle ne regardait
# rien, la seule configuration du dépôt vivant dans `app/`.
#
# ─────────────────────────────────────────────────────────────────────────────────────────────────
#  PÉRIMÈTRE INSPECTÉ
# ─────────────────────────────────────────────────────────────────────────────────────────────────
#
#  ELLE LIT  `app/**` et `web/**` en `.js`, `.ts`, `.vue`.
#
#  ELLE NE LIT PAS  `clients/` (généré depuis le contrat, principe I(a)), `backend/` (du Rust),
#  `docs/`, `infra/`, `specs/`, et les répertoires produits. Voir l'en-tête de `eslint.config.js`.
#
#  LIMITE ASSUMÉE  `(window as any).__TAURI__` échappe à `no-restricted-properties`, le cast
#  changeant la forme de l'expression dans l'arbre syntaxique. `no-explicit-any` l'attrape par un
#  autre chemin, mais un `eslint-disable` local suffirait à passer les deux. La revue reste
#  nécessaire sur ce cas précis.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

echo "── P-15 · 1/2 — ESLint sur app/ et web/ ──────────────────────────────────────────"

pnpm exec eslint . --max-warnings 0

echo "  ✓ aucune erreur, aucun avertissement"

echo "── P-15 · 2/2 — la porte a-t-elle une cible non vide dans CHAQUE arbre ? ─────────"

# Les trois arbres qui doivent être gardés, et le minimum attendu dans chacun.
#
# `web/qr` et `web/console` ne portent aujourd'hui qu'un `nuxt.config.ts` chacun — ce sont des
# coquilles, leurs écrans viennent aux cycles QRC et éditeur. **Un fichier est une cible non vide**,
# et c'est tout ce que l'exigence 4 demande : que la porte regarde quelque chose. Le jour où ces
# répertoires se remplissent, le compte monte tout seul.
node --input-type=module <<'JS'
import { loadESLint } from 'eslint'

const ARBRES = [
  { chemin: 'app', minimum: 10 },
  { chemin: 'web/qr', minimum: 1 },
  { chemin: 'web/console', minimum: 1 },
]

const ESLint = await loadESLint({ useFlatConfig: true })
const eslint = new ESLint()

let echec = false

for (const arbre of ARBRES) {
  const resultats = await eslint.lintFiles([arbre.chemin])
  // `lintFiles` renvoie un résultat par fichier RÉELLEMENT analysé — les fichiers ignorés n'y
  // figurent pas. C'est donc le décompte que la porte veut : ce qu'elle a lu, pas ce qui existe.
  const analyses = resultats.length

  if (analyses < arbre.minimum) {
    console.error(
      `  ✗ ${arbre.chemin} — ${analyses} fichier(s) analysé(s), ${arbre.minimum} attendu(s) au moins.\n`
      + '      Une porte dont la cible est vide est indistinguable d\'une porte qui passe\n'
      + '      (§ Couverture des portes, exigence 4). Causes probables : un motif `ignores` trop\n'
      + '      large dans eslint.config.js, ou un répertoire vidé sans que ce seuil soit revu.',
    )
    echec = true
    continue
  }

  const exemples = resultats.slice(0, 2).map(r => r.filePath.split('/').slice(-2).join('/'))
  console.log(`  ✓ ${arbre.chemin.padEnd(12)} ${String(analyses).padStart(3)} fichier(s) — ${exemples.join(', ')}…`)
}

if (echec) {
  console.error('')
  console.error('P-15 ÉCHOUE — la porte ne garde pas tout ce qu\'elle prétend garder.')
  process.exit(1)
}

console.log('── P-15 · bilan ──────────────────────────────────────────────────────────────────')
console.log('P-15 ✓ — pont natif confiné à app/core/platform/, et les deux surfaces HORS Tauri')
console.log('         (web/qr, web/console) sont désormais gardées elles aussi.')
JS
