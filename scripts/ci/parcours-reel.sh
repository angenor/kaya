#!/usr/bin/env bash
#
# **Porte P-22** — l'application démarre et chaque route déclarée s'atteint.
#
#     scripts/ci/parcours-reel.sh              exécute la porte
#     scripts/ci/parcours-reel.sh --negatif    prouve qu'elle sait échouer (exigence 4)
#
# ═════════════════════════════════════════════════════════════════════════════════════════════
#  CE QUE CETTE PORTE GARDE, ET QUE RIEN D'AUTRE NE GARDAIT
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
#  Le cycle 003 a été livré avec 24 portes vertes, 224 tests backend et 428 tests front — et deux
#  des quatre écrans du produit étaient inatteignables en navigateur. Les tests front montent les
#  composants avec `@vue/test-utils`, ce qui contourne le routeur, `<Suspense>`, les layouts et
#  les plugins : tout ce qui fait qu'une page existe pour un utilisateur.
#
#  Le périmètre exact, ce qu'elle ne lit pas, et la décision sur le styleguide sont écrits en tête
#  de `tests-e2e/parcours-reel.spec.ts`. Les reproduire ici en ferait une seconde copie qui
#  dériverait.
#
# ═════════════════════════════════════════════════════════════════════════════════════════════
#  PÉRIMÈTRE — LES MOTEURS, ET CE QU'ILS COUVRENT DU PRODUIT (exigence 1)
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
#  **La cible est Tauri, et Tauri n'embarque pas Chromium.** Il emprunte le moteur du système. La
#  correspondance décide de ce que cette porte couvre réellement :
#
#      Moteur du système                Cible du produit                Projet Playwright
#      ───────────────────────────────  ──────────────────────────────  ─────────────────
#      WebView2 (Chromium)              Windows                         chromium
#      Android System WebView (Chrom.)  Android                         chromium
#      WKWebView (WebKit)               macOS — le poste de dév.        webkit
#      WKWebView (WebKit)               iOS                             webkit
#      WebKitGTK (WebKit)               Linux                           webkit
#
#  Trois cibles sur cinq sont WebKit. Tant que la porte n'exerçait que Chromium, elle validait le
#  moteur que le produit n'utilise PAS sur la majorité de ses cibles — à commencer par celle sur
#  laquelle il est écrit. Les deux projets exécutent **les mêmes tests, sans exclusion** : un cas
#  qui tombe sous WebKit est un écart que la coquille Tauri rencontrera, pas un cas à retirer.
#
#  L'étape 3/5 ci-dessous ne se contente pas de l'écrire : elle **compte les projets que Playwright
#  exécutera vraiment** et refuse si l'un des deux manque. Une porte qui annonce deux moteurs et
#  n'en lance qu'un est indistinguable d'une porte qui passe (exigence 2).
#
#  CE QUE CELA NE COUVRE TOUJOURS PAS, et il faut le lire avant de conclure : le `webkit` de
#  Playwright **n'est pas WKWebView**. C'est une construction de WebKit maintenue par l'équipe
#  Playwright — plus proche de la cible que Chromium, et pas identique : ni les réglages du
#  composant système d'Apple, ni son intégration au processus hôte, ni les restrictions d'iOS. Le
#  contrôle réel de macOS et d'iOS viendra avec la coquille Tauri. Un vert ici dit « le produit
#  tourne sur un moteur WebKit », jamais « le produit est vérifié sur la cible ».
#
#  Firefox n'est pas couvert : aucune cible du produit n'emploie Gecko.
#
# ═════════════════════════════════════════════════════════════════════════════════════════════
#  CE QUE CETTE PORTE EXIGE DE L'ENVIRONNEMENT
# ═════════════════════════════════════════════════════════════════════════════════════════════
#
#  L'API, la base et les seeds. C'est le prix d'une porte qui exerce le produit plutôt qu'un
#  composant, et il est vérifié AVANT de lancer quoi que ce soit : une porte qui échoue pour une
#  raison d'environnement, sur un message illisible, finit par être ignorée — et une porte qu'on
#  ignore ne garde rien.
#
#  Playwright télécharge des navigateurs, mais il ne s'exécute jamais dans le produit : c'est de
#  l'outillage de développement, au même titre que `subset-font`. La porte P-21 ne porte que sur
#  ce que l'APPLICATION charge à l'exécution, et un navigateur rangé dans le cache du poste
#  n'entre ni dans le paquet Nuxt, ni dans l'image `linux/amd64`.

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

MODE_NEGATIF=0
[[ "${1:-}" == "--negatif" ]] && MODE_NEGATIF=1

echo "── P-22 · 1/5 — l'environnement ──────────────────────────────────────────────"

if [[ -f backend/.env ]]; then
    set -a
    # shellcheck disable=SC1091
    source backend/.env
    set +a
fi

if [[ -z "${KAYA_SEEDS_MOT_DE_PASSE:-}" ]]; then
    echo "  ✗ KAYA_SEEDS_MOT_DE_PASSE absent." >&2
    echo "    C'est la variable dont `seeds` se sert pour créer les comptes de démonstration ;" >&2
    echo "    aucun mot de passe n'est écrit dans le dépôt. Voir backend/.env.example." >&2
    exit 1
fi

# **Qui écoute réellement le port 8080 ?** Le contrôle vient AVANT la sonde de santé, et il ne
# dépend d'aucun nom de binaire. Au cycle 003, un `target/debug/api` d'une compilation antérieure —
# sous un nom qui n'existe plus au Cargo.toml — a répondu `/health` à la place du bon pendant toute
# une session. Une porte qui interroge un serveur périmé rend un verdict sur du code qui n'est plus
# là, ce qui est pire qu'un échec.
qui_ecoute=$(lsof -nP -iTCP:8080 -sTCP:LISTEN -Fn 2>/dev/null | head -20 || true)
if [[ -n "$qui_ecoute" ]]; then
    echo "  · port 8080 servi par :"
    lsof -nP -iTCP:8080 -sTCP:LISTEN 2>/dev/null | tail -n +2 | awk '{print "      " $1, "pid", $2}'
fi

if ! curl -sf -m 5 http://localhost:8080/health >/dev/null 2>&1; then
    echo "  ✗ l'API ne répond pas sur http://localhost:8080." >&2
    echo "    P-22 exerce le produit, pas un composant : il lui faut l'API, la base et les seeds." >&2
    echo >&2
    echo "      docker compose -f infra/compose.yml up -d" >&2
    echo "      bash scripts/dev/preparer-base.sh" >&2
    echo "      (cd backend && cargo run -p kaya-api --bin seeds)" >&2
    echo "      (cd backend && cargo run -p kaya-api --bin kaya-api)" >&2
    exit 1
fi
echo "  ✓ API joignable, mot de passe de démonstration défini"

echo "── P-22 · 2/5 — les routes inspectées ────────────────────────────────────────"
# Le décompte est imprimé AVANT l'exécution : une porte dont la cible a rétréci doit se voir ici,
# pas dans un résumé de fin où « 0 test, 0 échec » ressemble à un succès.
fichiers=$(find app/pages -maxdepth 1 -name '*.vue' | wc -l | tr -d ' ')
echo "  ${fichiers} page(s) dans app/pages/ :"
find app/pages -maxdepth 1 -name '*.vue' -exec basename {} \; | sort | sed 's/^/    /'
if [[ "$fichiers" -eq 0 ]]; then
    echo "  ✗ aucune page — la porte n'inspecterait rien et passerait au vert." >&2
    exit 1
fi

echo "── P-22 · 3/5 — les moteurs réellement exercés ───────────────────────────────"
#
# Exigence 2 : « compter les cibles réellement examinées et les comparer au total attendu ». La
# correspondance moteur ↔ cible est écrite en tête ; ici on vérifie qu'elle est TENUE. Un
# `projects:` amputé, un `--project=chromium` oublié dans un script d'appel, et la porte
# n'exercerait plus qu'un moteur en continuant d'afficher le même vert.
MOTEURS_ATTENDUS=(chromium webkit)
inventaire="$(pnpm exec playwright test --list 2>/dev/null || true)"

manquants=()
for moteur in "${MOTEURS_ATTENDUS[@]}"; do
    cas=$(printf '%s\n' "$inventaire" | grep -c "^  \[${moteur}\]" || true)
    if [[ "$cas" -eq 0 ]]; then
        manquants+=("$moteur")
        echo "  ✗ ${moteur} — AUCUN cas"
    else
        echo "  ✓ ${moteur} — ${cas} cas"
    fi
done

if [[ ${#manquants[@]} -gt 0 ]]; then
    echo "  ✗ moteur(s) annoncé(s) mais non exercé(s) : ${manquants[*]}" >&2
    echo "    La cible est Tauri, qui emprunte le moteur du système : WebKit couvre macOS, iOS" >&2
    echo "    et Linux, Chromium couvre Windows et Android. Un moteur absent retire trois cibles" >&2
    echo "    de la porte sans rien changer à son verdict." >&2
    echo "    Installer le navigateur manquant : pnpm exec playwright install ${manquants[*]}" >&2
    exit 1
fi

if [[ $MODE_NEGATIF -eq 1 ]]; then
    echo "── P-22 · 4/5 — TEST NÉGATIF : la porte sait-elle échouer ? ───────────────────"
    #
    # Exigence 4 du § « Couverture des portes » : « le test l'exerce sur au moins un cas réel ».
    # On casse une route pour de vrai — le layout perd son `<main>` —, on constate que la porte
    # refuse, puis on remet. Le `trap` garantit la remise même sur interruption : un contrôle qui
    # laisserait le dépôt modifié serait pire que pas de contrôle (exigence 3).
    #
    cible="app/layouts/default.vue"
    sauvegarde="$(mktemp)"
    cp "$cible" "$sauvegarde"
    remettre() { cp "$sauvegarde" "$cible"; rm -f "$sauvegarde"; echo "  ↩ $cible remis en l'état"; }
    trap remettre EXIT

    # `<main>` → `<div>` : la coquille perd son repère principal, sans rien casser d'autre.
    perl -0pi -e 's/<main class=/<div class=/; s/<\/main>/<\/div>/' "$cible"
    echo "  · $cible cassé volontairement — <main> remplacé par <div>"

    # Le filtre porte sur le **numéro** du contrôle, pas sur ses mots seuls : « 1 · chargement
    # DIRECT » désigne le contrôle par route et lui seul. Un filtre sur « chargement DIRECT »
    # attraperait aussi le contrôle 7 de la section « passer la main », qui ferme la session — il
    # tournerait alors sans que le contrôle 6 l'ait fermée, et son verdict dépendrait de la casse
    # d'un libellé. Une porte ne se pilote pas à la majuscule près.
    if pnpm exec playwright test --grep "1 · chargement DIRECT" >/dev/null 2>&1; then
        echo "  ✗ LA PORTE A PASSÉ SUR UNE COQUILLE CASSÉE." >&2
        echo "    Elle ne garde rien : le contrôle du <main> n'inspecte pas ce qu'il annonce." >&2
        exit 1
    fi
    echo "  ✓ la porte a refusé la coquille cassée"
    remettre
    trap - EXIT
    echo "── P-22 · 5/5 — test négatif concluant ───────────────────────────────────────"
    exit 0
fi

echo "── P-22 · 4/5 — le parcours réel ─────────────────────────────────────────────"
pnpm exec playwright test

echo "── P-22 · 5/5 — bilan ────────────────────────────────────────────────────────"
echo "P-22 ✓ — les ${fichiers} routes s'atteignent, en direct et par navigation, dans les deux"
echo "         thèmes, sur ${#MOTEURS_ATTENDUS[@]} moteurs (${MOTEURS_ATTENDUS[*]}), et la sortie de session"
echo "         renvoie sur /connexion par les deux mêmes chemins."
