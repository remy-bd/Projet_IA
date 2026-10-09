# ☢️ Apocalypse 3D - Moteur & Jeu Vulkan FPS (Rust)

Un jeu de tir à la première personne (FPS) 3D complet et optimisé, propulsé par un moteur graphique natif **Vulkan** en **Rust (wgpu)**.

---

## 📂 Structure du Projet (Explorateur)

```text
09_10_2026/
├── assets/
│   └── models/
│       ├── akm.obj          # Modèle 3D de l'AKM 7.62x39mm
│       ├── m4a1.obj         # Modèle 3D de la Carabine M4A1 5.56mm
│       ├── colt1911.obj     # Modèle 3D du Pistolet Colt 1911 .45 ACP
│       ├── enemy_bot.obj    # Modèle 3D des Bots ennemis
│       └── barrel.obj       # Modèle 3D des barils explosifs
├── src/
│   ├── assets/
│   │   ├── mod.rs
│   │   └── models.rs        # Générateur & parseur de modèles 3D (.obj)
│   ├── audio/
│   │   ├── mod.rs
│   │   └── sounds.rs        # Moteur audio procédural (tirs, rechargements, impacts, explosions)
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── camera.rs        # Caméra FPS (recul dynamique, visée ADS, secousses)
│   │   ├── mesh.rs          # Gestion des tampons GPU (Vertex/Index buffers)
│   │   └── renderer.rs      # Pipeline graphique Vulkan natif (Depth Stencil, Uniforms)
│   ├── game/
│   │   ├── mod.rs
│   │   ├── bot.rs           # IA des Bots (patrouille, visée, tir, couverture, ragdoll)
│   │   ├── hud.rs           # Interface tactique 2D/3D (réticule dynamique, vie, munitions, radar)
│   │   ├── map.rs           # Map apocalyptique, collisions AABB, 6 spawns aléatoires
│   │   ├── particles.rs     # Traceurs de balles, étincelles, sang et explosions
│   │   └── player.rs        # Contrôleur joueur FPS, physique, inventaire, tirs
│   ├── shaders/
│   │   └── main.wgsl        # Shader 3D (lumière directionnelle, spéculaire, brouillard, flashs)
│   └── main.rs              # Boucle principale de jeu & gestionnaire d'événements
├── Cargo.toml               # Configuration des dépendances Rust
└── README.md                # Documentation complète
```

---

## 🎮 Commandes de Jeu

| Action | Touche / Contrôle |
|---|---|
| **Avancer** | <kbd>Z</kbd> ou <kbd>W</kbd> |
| **Reculer** | <kbd>S</kbd> |
| **Gauche** | <kbd>Q</kbd> ou <kbd>A</kbd> |
| **Droite** | <kbd>D</kbd> |
| **Sauter** | <kbd>Espace</kbd> |
| **S'accroupir** | <kbd>Ctrl</kbd> |
| **Sprinter** | <kbd>Shift</kbd> |
| **Tirer** | <kbd>Clic Gauche</kbd> |
| **Viser (ADS)** | <kbd>Clic Droit</kbd> (maintien) |
| **Recharger** | <kbd>R</kbd> |
| **Arme 1 (AKM)** | <kbd>1</kbd> |
| **Arme 2 (M4A1)** | <kbd>2</kbd> |
| **Arme 3 (Colt 1911)** | <kbd>3</kbd> |
| **Changer d'arme** | <kbd>Molette Souris</kbd> |
| **Libérer la souris / Pause** | <kbd>Échap</kbd> |

---

## ⚡ Caractéristiques Principales

1. **Moteur 3D Vulkan Natif** :
   - Pipeline avec `wgpu` ciblant l'API **Vulkan**.
   - Éclairage directionnel (soleil apocalyptique), reflets spéculaires Blinn-Phong et éclairage ponctuel dynamique lors des tirs.
   - Système de brouillard volumétrique d'ambiance post-apocalyptique.

2. **Arsenal Réaliste (.OBJ 3D)** :
   - **AKM (7.62x39mm)** : Forte puissance, recul marqué, chargeur 30 coups, automatique.
   - **M4A1 (5.56mm NATO)** : Cadence élevée, précision accrue, chargeur 30 coups, automatique.
   - **Colt 1911 (.45 ACP)** : Dégâts lourds par coup, tir semi-automatique percutant, chargeur 7 coups.
   - Animations de recul, mouvement d'inertie (sway), balancement de marche (bobbing) et visée épaulée (ADS).
   - Rechargements animés avec sons synchronisés (éjection chargeur, insertion, réarmement de la culasse).

3. **Map Urbaine Apocalyptique & 6 Spawns Aléatoires** :
   - Bâtiments en ruines multi-niveaux, carcasses de voitures, conteneurs maritimes, barricades et sacs de sable.
   - 14 barils explosifs interactifs provoquant des dégâts de zone (AoE).
   - 6 points d'apparition tactiques aléatoires pour le joueur et les bots.
   - Système de collisions physiques AABB temps réel.

4. **Bots Ennemis Armés & IA Réactive** :
   - Détection par champ de vision et raycasting.
   - Patrouille, poursuite, mise à couvert et tirs en rafales.
   - Équipés d'armes aléatoires (AKM, M4A1, Colt 1911).
   - Système de points de vie, animation de chute à la mort et récompense en munitions lors des éliminations.

5. **Audio Procédural Intégré** :
   - Synthèse sonore PCM temps réel pour les tirs, rechargements, bruits de pas, explosions et impacts.

---

## 🚀 Lancement

Pour lancer le jeu en mode développement :
```powershell
cargo run
```

Pour lancer avec les optimisations maximales (Release 120+ FPS) :
```powershell
cargo run --release
```
