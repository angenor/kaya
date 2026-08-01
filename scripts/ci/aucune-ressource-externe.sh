#!/usr/bin/env bash
#
# Porte P-21 — aucune ressource chargée depuis un hôte externe.
#
#     pnpm porte:p21
#
# Police, icône, script, feuille de style, image : tout est embarqué dans l'application. Un CDN
# rend l'écran dépendant du réseau, ce que le principe VI interdit. Le principe XII disait déjà de
# réimplémenter le HTML de maquette sans rien dire des **ressources qu'il charge** — c'est le trou
# que le cycle 002 a laissé passer, et que cette porte ferme.
#
# ─────────────────────────────────────────────────────────────────────────────────────────────────
#  PÉRIMÈTRE INSPECTÉ — ce que la porte lit, et ce qu'elle ne lit pas
# ─────────────────────────────────────────────────────────────────────────────────────────────────
#
#  ELLE LIT
#    · `app/`  — l'application unique : `.vue`, `.ts`, `.js`, `.css`, `.html`, `.json`
#    · `web/`  — les deux surfaces publiques (page QR, console éditeur), mêmes extensions
#    · toute feuille `.css` de ces deux arbres, avec une règle PLUS STRICTE : dans une feuille,
#      une URL absolue est toujours une ressource, quelle que soit sa forme.
#
#  ELLE NE LIT PAS, et c'est délibéré
#    · `docs/design/` — les maquettes chargent VOLONTAIREMENT Tailwind, Google Fonts et Phosphor
#      depuis un CDN. Ce sont des cibles de lecture, pas du code livré (principe XII), et leur
#      README le dit. Les soumettre à cette porte reviendrait à interdire la référence visuelle.
#    · `backend/` — aucune surface d'affichage.
#    · `node_modules/`, `.nuxt/`, `.output/`, `dist/`, `src-tauri/` — construits ou tiers.
#    · les URL **construites à l'exécution** par concaténation. Limite réelle : un
#      `'https://' + hote` échapperait à l'analyse. Aucune n'existe aujourd'hui ; la revue couvre.
#
#  TROIS CONTRÔLES
#    1. Formes de chargement de ressource — `src=`, `href=`, `srcset=`, `poster=`, `url(`,
#       `@import`, `import … from` — pointant un hôte hors liste blanche.
#    2. Feuilles de style — toute URL absolue hors liste blanche, sans condition de forme.
#    3. Cible non vide — un décompte de zéro fichier est un échec, pas un succès (constitution,
#       § Couverture des portes, exigence 4).
#
#  CE QUE CETTE PORTE NE FAIT PLUS, ET OÙ C'EST PASSÉ
#    Le contrôle « les icônes employées sont réellement embarquées » vivait ici comme 4e contrôle.
#    Il est **déplacé** — pas dupliqué — dans `scripts/ci/ressources-embarquees.sh`, la porte
#    **P-21b**, avec ses deux frères : les familles de `--font-*` servies en local, et U+202F lu
#    dans la table `cmap`. Une interdiction (P-21, « rien d'externe ») et son versant positif
#    (P-21b, « le local existe ») se lisent mieux séparés, et deux copies du même contrôle
#    finissent par diverger. **P-21 sans P-21b reste franchissable en supprimant la cible.**
#
#  LISTE BLANCHE, nommée et motivée
#    · `localhost`, `127.0.0.1`, `0.0.0.0`, `::1` — développement local. L'API n'est pas une
#      ressource de page : c'est le service que l'application appelle, et son adresse vit dans
#      `runtimeConfig`, jamais dans un composant.
#    · `www.w3.org` — espace de nom XML/SVG (`xmlns`). Jamais chargé par un navigateur.
#    · schémas non réseau — `data:`, `blob:`, `about:`, `tauri:`.
#
#  La porte NE MODIFIE RIEN de ce qu'elle inspecte (exigence 3).

set -euo pipefail

racine="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$racine"

python3 - <<'PY'
import os
import re
import sys

RACINE = os.getcwd()
ARBRES = ["app", "web"]
EXTENSIONS = (".vue", ".ts", ".js", ".mjs", ".css", ".html", ".json")
IGNORES = {"node_modules", ".nuxt", ".output", "dist", "src-tauri", ".git"}

# Hôtes et schémas admis — voir l'en-tête du script pour le motif de chacun.
HOTES_ADMIS = {"localhost", "127.0.0.1", "0.0.0.0", "::1", "[::1]", "www.w3.org"}
SCHEMAS_ADMIS = ("data:", "blob:", "about:", "tauri:")

echec = False


def signaler(message):
    global echec
    print(f"  ✗ {message}", file=sys.stderr)
    echec = True


def sans_commentaires(contenu):
    """Un commentaire ne charge rien. Le `[^:]` protège les `https://` d'un `//` de fin de ligne."""
    contenu = re.sub(r"/\*[\s\S]*?\*/", "", contenu)
    contenu = re.sub(r"(^|[^:])//.*$", r"\1", contenu, flags=re.M)
    contenu = re.sub(r"<!--[\s\S]*?-->", "", contenu)
    return contenu


def hote_de(url):
    sans_schema = re.sub(r"^[a-zA-Z][a-zA-Z0-9+.-]*:", "", url)
    return sans_schema.lstrip("/").split("/")[0].split("?")[0].split("#")[0].split(":")[0]


def admis(url):
    if url.startswith(SCHEMAS_ADMIS):
        return True
    return hote_de(url) in HOTES_ADMIS


def fichiers(arbre):
    for repertoire, sous, noms in os.walk(os.path.join(RACINE, arbre)):
        sous[:] = [d for d in sous if d not in IGNORES]
        for nom in noms:
            if nom.endswith(EXTENSIONS):
                yield os.path.join(repertoire, nom)


# Une URL absolue : `https://…`, `http://…`, ou protocole-relatif `//hote/…`.
URL_ABSOLUE = r"(?:https?:)?//[^\s\"'`)>]+"

# Les formes qui CHARGENT une ressource. Une URL passée à un client d'API n'en est pas une —
# c'est la distinction que la porte tient, et sa limite est écrite en tête.
FORMES = [
    ("attribut src/href/srcset/poster", re.compile(
        r"""\b(?:src|href|srcset|poster|data-src)\s*=\s*["'`]?\s*(""" + URL_ABSOLUE + r""")""")),
    ("url() CSS", re.compile(r"""url\(\s*["']?\s*(""" + URL_ABSOLUE + r""")""")),
    ("@import", re.compile(r"""@import\s+(?:url\()?\s*["']?\s*(""" + URL_ABSOLUE + r""")""")),
    ("import de module", re.compile(
        r"""\bimport\s*(?:\(|[\s\S]{0,80}?\bfrom\s*)["'`](""" + URL_ABSOLUE + r""")""")),
]

# =================================================================================================
#  1 et 2 — formes de chargement, et règle stricte sur les feuilles de style
# =================================================================================================

print("── P-21 · 1/3 — ressources déclarées dans app/ et web/ "
      + "─" * 22)

inspectes = 0
feuilles = 0
urls_examinees = 0

for arbre in ARBRES:
    for chemin in sorted(fichiers(arbre)):
        relatif = os.path.relpath(chemin, RACINE)
        inspectes += 1
        contenu = sans_commentaires(open(chemin, encoding="utf-8", errors="replace").read())

        for nom_forme, motif in FORMES:
            for capture in motif.finditer(contenu):
                urls_examinees += 1
                url = capture.group(1)
                if admis(url):
                    continue
                ligne = contenu[: capture.start()].count("\n") + 1
                signaler(
                    f"{relatif}:{ligne} — {nom_forme} vers un hôte externe : « {url[:90]} »\n"
                    "      Tout est embarqué dans l'application. Un CDN rend l'écran dépendant\n"
                    "      du réseau, ce que le mode hors-ligne interdit (principe VI, porte P-21)."
                )

        if chemin.endswith(".css"):
            feuilles += 1
            for capture in re.finditer(URL_ABSOLUE, contenu):
                urls_examinees += 1
                url = capture.group(0)
                if admis(url):
                    continue
                ligne = contenu[: capture.start()].count("\n") + 1
                signaler(
                    f"{relatif}:{ligne} — URL absolue dans une feuille de style : « {url[:90]} »\n"
                    "      Dans une feuille, une URL est TOUJOURS une ressource : police, image,\n"
                    "      feuille importée. Aucune ne vient d'un hôte externe (porte P-21)."
                )

print(f"  {inspectes} fichier(s) inspecté(s), dont {feuilles} feuille(s) de style")
print(f"  {urls_examinees} URL absolue(s) examinée(s)")

# =================================================================================================
#  3 — la porte a-t-elle une cible ?
# =================================================================================================

print("── P-21 · 2/3 — la porte a-t-elle une cible non vide ? "
      + "─" * 27)

# « Une porte dont la cible est vide est indistinguable d'une porte qui passe » (constitution,
# § Couverture des portes). Les seuils sont bas exprès : ils attrapent le cas où un renommage de
# répertoire aveugle la porte, pas une variation de contenu.
if inspectes == 0:
    signaler("aucun fichier inspecté — la porte ne garde RIEN. Vérifier les chemins `app/` et `web/`.")
elif feuilles == 0:
    signaler("aucune feuille de style inspectée — c'est là que vivent @font-face et url().")
else:
    print(f"  ✓ cible non vide : {inspectes} fichier(s), {feuilles} feuille(s)")

# =================================================================================================

print("── P-21 · 3/3 — bilan " + "─" * 52)

if echec:
    print("", file=sys.stderr)
    print("P-21 ÉCHOUE — aucune ressource ne vient d'un hôte externe : police, icône, script,",
          file=sys.stderr)
    print("feuille de style, image. Tout est embarqué (principe VI, principe XII).", file=sys.stderr)
    sys.exit(1)

print("P-21 ✓ — aucune ressource d'hôte externe : police, icône, script, feuille, image.")
print("  Le versant positif — ce qui est déclaré est effectivement embarqué — est la porte")
print("  P-21b, `scripts/ci/ressources-embarquees.sh`. P-21 seule reste franchissable en")
print("  supprimant la cible : c'est ce qui a produit un écran sans icônes au cycle 002.")
print("  Limite assumée : les URL construites à l'exécution par concaténation échappent à")
print("  l'analyse. Aucune n'existe aujourd'hui ; la revue couvre ce cas (voir l'en-tête).")
PY
