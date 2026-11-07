const project = structuredClone(window.SAMPLE);
const state = {
  view: "passages",
  passage: null,
  map: "field",
  npc: "mira",
  item: "sword",
  stat: "hp",
  skill: "steady",
  folder: null,
  tile: { x: 93, y: 9 },
  tool: "wall",
  paintNpc: "slime",
  modal: null,
  buildParent: "",
  buildName: "field-game",
  savePath: "field-and-town.fable.json",
  notice: "",
  artOwner: "Mira",
  artRole: "overworld",
};

const NAV = [
  ["passages", "Passages"],
  ["maps", "Maps"],
  ["npcs", "NPCs"],
  ["items", "Items"],
  ["stats", "Stats"],
  ["skills", "Skills"],
  ["art", "Art"],
];

function esc(value) {
  return String(value ?? "").replace(/[&<>"']/g, (ch) => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;",
  }[ch]));
}

function byId(list, id) {
  return list.find((item) => item.id === id);
}

function mapOf() {
  return byId(project.maps, state.map) || project.maps[0];
}

function applyHash() {
  const hash = location.hash.replace("#", "");
  if (hash === "build") {
    state.view = "passages";
    state.modal = "build";
  } else if (hash === "map") state.view = "maps";
  else if (hash === "npc") state.view = "npcs";
  else if (hash === "skill") state.view = "skills";
  else if (NAV.some(([id]) => id === hash)) state.view = hash;
}

function chip(check) {
  if (check.kind === "flag") {
    const label = check.want ? check.flag : "!" + check.flag;
    return `<span class="chip ${check.want ? "" : "off"}">${esc(label)}</span>`;
  }
  if (check.kind === "stat") {
    return `<span class="chip">${esc(check.stat)} ${esc(check.cmp)} ${esc(check.value)}</span>`;
  }
  if (check.kind === "combat") {
    return `<span class="chip">${check.win ? "combat win" : "combat loss"}</span>`;
  }
  return "";
}

function passageClass(passage) {
  if (passage.kind === "terminus") return "terminus";
  if (passage.kind === "place") return "place";
  if (passage.id === "fight" || passage.slot) return "fight";
  if (passage.id === "peace") return "peace";
  return "passage";
}

function placeCards() {
  return project.npcs.filter((npc) => npc.place).map((npc, index) => ({
    id: "place-" + npc.id,
    kind: "place",
    text: `${npc.place.map} ${npc.place.x}, ${npc.place.y}`,
    x: npc.place.ux ?? 36,
    y: npc.place.uy ?? 340 + index * 150,
    checks: npc.place.checks || [],
    title: npc.name + " stands",
    npcId: npc.id,
  }));
}

function allCards() {
  const passages = project.passages.map((passage) => ({
    ...passage,
    title: passage.id,
  }));
  return passages.concat(placeCards());
}

function renderList() {
  if (state.view === "passages") {
    const items = allCards().map((card) => {
      const on = state.passage === card.id ? "on" : "";
      return `<button class="row ${on}" data-act="pick-passage" data-id="${esc(card.id)}">${esc(card.title || card.id)}</button>`;
    }).join("");
    return `<h2>Passages</h2>${items}<button class="row add" data-act="add-passage">Add passage</button>`;
  }
  if (state.view === "maps") {
    const items = project.maps.map((map) => {
      const on = state.map === map.id ? "on" : "";
      return `<button class="row ${on}" data-act="pick-map" data-id="${esc(map.id)}">${esc(map.name)}</button>`;
    }).join("");
    return `<h2>Maps</h2>${items}`;
  }
  if (state.view === "npcs" || state.view === "items") {
    const kind = state.view === "npcs" ? "npc" : "item";
    const folders = project.folders.filter((folder) => folder.kind === kind);
    let html = `<h2>Folders</h2>`;
    for (const folder of folders) {
      const on = state.folder === folder.id ? "on" : "";
      html += `<button class="row folder ${on}" data-act="pick-folder" data-id="${esc(folder.id)}">${esc(folder.name)}</button>`;
      const rows = kind === "npc"
        ? project.npcs.filter((npc) => npc.folder === folder.id)
        : project.items.filter((item) => item.folder === folder.id);
      for (const row of rows) {
        const selected = kind === "npc" ? state.npc === row.id : state.item === row.id;
        const act = kind === "npc" ? "pick-npc" : "pick-item";
        html += `<button class="row ${selected && !state.folder ? "on" : ""}" data-act="${act}" data-id="${esc(row.id)}">${esc(row.name)}</button>`;
      }
      const add = kind === "npc" ? "add-npc" : "add-item";
      html += `<button class="row add" data-act="${add}" data-id="${esc(folder.id)}">Add</button>`;
    }
    return html;
  }
  if (state.view === "stats") {
    const items = project.stats.map((stat) => {
      const on = state.stat === stat.id ? "on" : "";
      return `<button class="row ${on}" data-act="pick-stat" data-id="${esc(stat.id)}">${esc(stat.name)}</button>`;
    }).join("");
    return `<h2>Stats</h2>${items}<button class="row add" data-act="add-stat">Add stat</button>`;
  }
  if (state.view === "skills") {
    const items = project.skills.map((skill) => {
      const on = state.skill === skill.id ? "on" : "";
      return `<button class="row ${on}" data-act="pick-skill" data-id="${esc(skill.id)}">${esc(skill.name)}</button>`;
    }).join("");
    return `<h2>Skills</h2>${items}<button class="row add" data-act="add-skill">Add node</button>`;
  }
  return `<h2>Art</h2><p class="row">Kenney sheet</p>`;
}

function renderPassageBoard() {
  const cards = allCards().map((card) => {
    const reqs = (card.checks || []).map(chip).join("");
    const choices = (card.choices || []).map((choice) => {
      const sets = (choice.sets || []).length ? `<small>sets ${esc(choice.sets.join(", "))}</small>` : "";
      return `<button class="choice" data-goto="${esc(choice.goto)}" data-fail="${esc(choice.fail || "")}">${esc(choice.label)}${sets}</button>`;
    }).join("");
    const slot = card.slot
      ? `<p class="slot">minigame ${esc(card.slot)}${card.battle ? " · " + esc(card.battle.entity) + " on " + esc(card.battle.map) : ""}</p>`
      : "";
    const fail = card.fail
      ? `<button class="failport" data-fail="${esc(card.fail)}">failure</button>`
      : "";
    const body = card.text ? `<p class="body">${esc(card.text)}</p>` : "";
    const on = state.passage === card.id ? "on" : "";
    return `<article class="node ${passageClass(card)} ${on}" data-id="${esc(card.id)}" data-npc="${esc(card.npcId || "")}" style="left:${card.x || 0}px;top:${card.y || 0}px">
      <header>${esc(card.title || card.id)}</header>
      ${reqs ? `<div class="reqbox" data-port="in">${reqs}</div>` : `<div class="reqbox" data-port="in"></div>`}
      ${body}${slot}${choices}${fail}
    </article>`;
  }).join("");
  return `<div class="board" id="board">${cards}<svg></svg><div class="key"><span><i class="swatch ok"></i>success</span><span><i class="swatch bad"></i>failure</span></div></div>${state.passage ? renderDock() : ""}`;
}

function renderDock() {
  const passage = project.passages.find((item) => item.id === state.passage);
  if (!passage) return "";
  const options = allCards().map((card) => `<option value="${esc(card.id)}">${esc(card.title || card.id)}</option>`).join("");
  const choices = (passage.choices || []).map((choice, index) => `
    <label class="field"><span>Choice</span>
      <input data-choice="${index}" data-key="label" value="${esc(choice.label)}">
    </label>
    <label class="field"><span>Success</span>
      <input data-choice="${index}" data-key="goto" value="${esc(choice.goto)}">
    </label>
    <label class="field"><span>Failure</span>
      <input data-choice="${index}" data-key="fail" value="${esc(choice.fail || "")}">
    </label>
    <label class="field"><span>Sets flags</span>
      <input data-choice="${index}" data-key="sets" value="${esc((choice.sets || []).join(" "))}">
    </label>
    <button class="btn" data-act="add-choice-check" data-index="${index}" data-check="flag">Require a flag</button>
    <button class="btn" data-act="add-choice-check" data-index="${index}" data-check="stat">Require a stat</button>
    <button class="btn" data-act="add-choice-check" data-index="${index}" data-check="combat">Require combat</button>
  `).join("");
  return `<div class="dock">
    <h3>${esc(passage.id)}</h3>
    <label class="field"><span>Text</span><textarea data-field="passage-text">${esc(passage.text || "")}</textarea></label>
    <label class="field"><span>Minigame slot</span>
      <select data-field="passage-slot">
        <option value="" ${passage.slot ? "" : "selected"}>none</option>
        ${project.slots.map((slot) => `<option ${passage.slot === slot.name ? "selected" : ""}>${esc(slot.name)}</option>`).join("")}
      </select>
    </label>
    <p class="lede">A line runs from a choice's success to the requirement on the next node. The other line is the failure.</p>
    <button class="btn" data-act="add-node-check" data-check="flag">Add flag requirement</button>
    <button class="btn" data-act="add-node-check" data-check="stat">Add stat requirement</button>
    <button class="btn" data-act="add-node-check" data-check="combat">Add combat requirement</button>
    <button class="btn" data-act="add-choice">Add choice</button>
    ${choices}
    <p class="meta">Targets ${options ? "include the nodes on the map" : ""}.</p>
  </div>`;
}

function tileChar(map, x, y) {
  const rows = map.grid.split("\n").filter((row) => row.length);
  const row = rows[y] || "";
  return row[x] || ".";
}

function tileKind(map, x, y) {
  const npc = project.npcs.find((item) => item.place && item.place.map === map.id && item.place.x === x && item.place.y === y);
  if (npc) return { kind: "NPC", title: npc.name, detail: "Stands on this cell under the flags on its place node." };
  const entity = (map.entities || []).find((item) => item.x === x && item.y === y);
  if (entity) return { kind: "Entity", title: entity.name, detail: "An entity such as a chest, sitting on the floor." };
  const ch = tileChar(map, x, y);
  if ("RWB S#".includes(ch) || "RWSB#".includes(ch)) {
    const names = { R: "Red roof", W: "Wood wall", B: "Blue roof", S: "Stone wall", "#": "Void" };
    return { kind: "Wall", title: names[ch] || "Wall", detail: "You cannot pass this cell." };
  }
  const floors = { ".": "Grass", "<": "Road edge", "=": "Road", ">": "Road edge", "*": "Imported tile" };
  return { kind: "Floor", title: floors[ch] || "Floor", detail: "You can walk here." };
}

function renderMap() {
  const map = mapOf();
  const here = tileKind(map, state.tile.x, state.tile.y);
  const tools = ["floor", "wall", "npc", "entity"].map((tool) => {
    const label = tool === "npc" ? "NPC" : tool[0].toUpperCase() + tool.slice(1);
    return `<button class="${state.tool === tool ? "on" : ""}" data-act="tool" data-id="${tool}">${label}</button>`;
  }).join("");
  return `<div class="mapview"><div class="tools">${tools}
      <label class="inline">NPC
        <select data-field="paint-npc">${project.npcs.map((npc) => `<option ${state.paintNpc === npc.id ? "selected" : ""}>${esc(npc.id)}</option>`).join("")}</select>
      </label>
    </div>
    <div class="mapstage">
      <canvas class="grid" id="map" width="${map.width * 7}" height="${map.height * 7}"></canvas>
      <div class="inspector">
        <h1>${esc(map.name)}</h1>
        <p class="lede">Tile ${state.tile.x}, ${state.tile.y}</p>
        <div class="kind ${here.kind.toLowerCase()}">${esc(here.kind)}</div>
        <h3>${esc(here.title)}</h3>
        <p>${esc(here.detail)}</p>
        <p class="lede">Floor is grass and road. A wall cannot be passed. An NPC stands on a cell. An entity, such as a chest, sits on the floor.</p>
      </div>
    </div></div>`;
}

function renderNpc() {
  if (state.folder) return renderFolder();
  const npc = byId(project.npcs, state.npc);
  if (!npc) return `<div class="form wide"><h1>NPCs</h1></div>`;
  const stats = project.stats.map((stat) => {
    const value = npc.bases[stat.id] ?? 0;
    return `<label class="stat"><span>${esc(stat.name)}</span><input type="number" data-base="${esc(stat.id)}" value="${esc(value)}"></label>`;
  }).join("");
  const place = npc.place
    ? `<h3>Stands</h3>
       <div class="inline">
         <input data-field="place-map" value="${esc(npc.place.map)}">
         <input type="number" data-field="place-x" value="${esc(npc.place.x)}">
         <input type="number" data-field="place-y" value="${esc(npc.place.y)}">
       </div>
       <div class="reqbox">${(npc.place.checks || []).map(chip).join("")}</div>`
    : `<p class="lede">The player starts at ${esc(mapOf().id)} ${mapOf().startX}, ${mapOf().startY}. A new game uses these base numbers.</p>`;
  return `<div class="form wide">
    <h1>${esc(npc.name)}</h1>
    <p class="lede">${npc.player ? "Player character" : "NPC"} · ${esc(npc.folder)}</p>
    <label class="field"><span>Name</span><input data-field="npc-name" value="${esc(npc.name)}"></label>
    <label class="field"><span>Folder</span>
      <select data-field="npc-folder">${project.folders.filter((folder) => folder.kind === "npc").map((folder) => `<option ${npc.folder === folder.id ? "selected" : ""} value="${esc(folder.id)}">${esc(folder.name)}</option>`).join("")}</select>
    </label>
    <label class="inline"><input type="checkbox" data-field="npc-player" ${npc.player ? "checked" : ""}> Player</label>
    <h3>Base stats</h3>
    <div class="stats">${stats}</div>
    <h3>Gear</h3>
    <label class="field"><span>Weapon</span><input data-field="npc-weapon" value="${esc(npc.weapon || "")}"></label>
    <label class="field"><span>Weapon attack</span><input type="number" data-field="npc-weapon-atk" value="${esc(npc.weaponAtk || 0)}"></label>
    ${place}
  </div>`;
}

function renderItem() {
  if (state.folder) return renderFolder();
  const item = byId(project.items, state.item);
  if (!item) return `<div class="form wide"><h1>Items</h1></div>`;
  const mods = (list, key) => (list || []).map((mod, index) => `
    <div class="inline">
      <select data-mod="${key}" data-index="${index}" data-key="stat">${project.stats.map((stat) => `<option ${mod.stat === stat.id ? "selected" : ""}>${esc(stat.id)}</option>`).join("")}</select>
      <input type="number" data-mod="${key}" data-index="${index}" data-key="delta" value="${esc(mod.delta)}">
    </div>`).join("");
  return `<div class="form wide">
    <h1>${esc(item.name)}</h1>
    <p class="lede">${esc(item.folder)}</p>
    <label class="field"><span>Name</span><input data-field="item-name" value="${esc(item.name)}"></label>
    <label class="field"><span>Folder</span>
      <select data-field="item-folder">${project.folders.filter((folder) => folder.kind === "item").map((folder) => `<option ${item.folder === folder.id ? "selected" : ""} value="${esc(folder.id)}">${esc(folder.name)}</option>`).join("")}</select>
    </label>
    <h3>When equipped</h3>
    ${mods(item.equip, "equip")}
    <button class="btn" data-act="add-mod" data-id="equip">Add gear bonus</button>
    <h3>When used</h3>
    ${mods(item.useMods, "use")}
    <button class="btn" data-act="add-mod" data-id="use">Add use effect</button>
  </div>`;
}

function renderFolder() {
  const folder = byId(project.folders, state.folder);
  if (!folder) return "";
  return `<div class="form wide">
    <h1>${esc(folder.name)}</h1>
    <p class="lede">Folder of ${esc(folder.kind === "npc" ? "NPCs" : "items")}</p>
    <label class="field"><span>Name</span><input data-field="folder-name" value="${esc(folder.name)}"></label>
  </div>`;
}

function renderStats() {
  const stat = byId(project.stats, state.stat) || project.stats[0];
  return `<div class="form wide">
    <h1>Stats</h1>
    <p class="lede">A named list. Adding a stat adds it to the player and every NPC. A new game starts from each NPC's base numbers.</p>
    <label class="field"><span>Name</span><input data-field="stat-name" value="${esc(stat.name)}"></label>
    <label class="field"><span>Id</span><input value="${esc(stat.id)}" disabled></label>
    <h3>Bases</h3>
    ${project.npcs.map((npc) => `<p>${esc(npc.name)} · ${esc(stat.name)} ${esc(npc.bases[stat.id] ?? 0)}</p>`).join("")}
  </div>`;
}

function renderSkills() {
  const cards = project.skills.map((skill) => {
    const needs = [
      ...(skill.needStats || []).map((check) => `${check.stat} ${check.cmp} ${check.value}`),
      ...(skill.needNodes || []).map((id) => "node " + ((byId(project.skills, id) || {}).name || id)),
    ];
    const cost = (skill.cost || []).map((mod) => `${mod.delta} ${mod.stat}`).join(", ") || "free";
    return `<article class="skillnode ${state.skill === skill.id ? "on" : ""}" data-skill="${esc(skill.id)}" style="left:${skill.x}px;top:${skill.y}px">
      <header>${esc(skill.name)}</header>
      <p>costs ${esc(cost)}</p>
      <p>${esc(needs.join(" · ") || "no requirement")}</p>
      <p>grants ${esc(skill.name)}</p>
    </article>`;
  }).join("");
  const skill = byId(project.skills, state.skill);
  const dock = skill ? `<div class="dock">
      <h3>${esc(skill.name)}</h3>
      <label class="field"><span>Grants</span><input data-field="skill-name" value="${esc(skill.name)}"></label>
      <p class="lede">Costs ${(skill.cost || []).map((mod) => esc(mod.delta + " " + mod.stat)).join(", ") || "nothing"}.</p>
      <button class="btn" data-act="add-skill-cost">Add a stat cost</button>
      <button class="btn" data-act="add-skill-stat">Require a stat</button>
      <button class="btn" data-act="add-skill-node">Require the other node</button>
    </div>` : "";
  return `<div class="board" id="skillboard">${cards}<svg></svg></div>${dock}`;
}

function renderArt() {
  const sprites = (project.art.sprites || []).map((sprite) => `<p>${esc(sprite.owner)} · ${esc(sprite.role)} · ${esc(sprite.frames)} frames</p>`).join("");
  return `<div class="form wide art">
    <h1>Art</h1>
    <p class="lede">The Kenney sheet is the sample. Import tiles, an overworld sprite, or a combat strip of frames.</p>
    <img src="kenney.png" alt="Kenney tile and sprite sheet">
    <label class="field"><span>Tiles</span><input type="file" accept="image/png" data-import="tiles"></label>
    <div class="inline">
      <label class="field"><span>Sprite for</span><input data-field="art-owner" value="${esc(state.artOwner)}"></label>
      <label class="field"><span>Role</span>
        <select data-field="art-role">
          <option ${state.artRole === "overworld" ? "selected" : ""}>overworld</option>
          <option ${state.artRole === "combat" ? "selected" : ""}>combat</option>
        </select>
      </label>
    </div>
    <label class="field"><span>Frame strip</span><input type="file" accept="image/png" data-import="sprite"></label>
    ${sprites || "<p class='lede'>No imported sprites yet. The game keeps the Kenney heroes.</p>"}
  </div>`;
}

function renderWork() {
  if (state.view === "passages") return renderPassageBoard();
  if (state.view === "maps") return renderMap();
  if (state.view === "npcs") return renderNpc();
  if (state.view === "items") return renderItem();
  if (state.view === "stats") return renderStats();
  if (state.view === "skills") return renderSkills();
  return renderArt();
}

function renderModal() {
  if (state.modal === "build") {
    return `<div class="modal"><form class="dialog" data-act="build-submit">
      <h1>Build</h1>
      <p class="lede">Choose a parent folder and a new folder name. The portable game is written there: the pixel exe and the project data, with nothing to install.</p>
      <label class="field"><span>Parent folder</span><input id="build-parent" data-field="build-parent" placeholder="/home/you/Games" value="${esc(state.buildParent)}" autofocus></label>
      <label class="field"><span>New folder name</span><input data-field="build-name" value="${esc(state.buildName)}"></label>
      <div class="rowbtns">
        <button class="btn" type="button" data-act="close-modal">Cancel</button>
        <button class="btn primary" type="submit">Build</button>
      </div>
    </form></div>`;
  }
  if (state.modal === "save") {
    return `<div class="modal"><form class="dialog" data-act="save-submit">
      <h1>Save</h1>
      <p class="lede">This stores the author project only. It does not build the game.</p>
      <label class="field"><span>File</span><input data-field="save-path" value="${esc(state.savePath)}"></label>
      <div class="rowbtns">
        <button class="btn" type="button" data-act="close-modal">Cancel</button>
        <button class="btn primary" type="submit">Save</button>
      </div>
    </form></div>`;
  }
  return "";
}

function render() {
  const nav = NAV.map(([id, label]) => `<button class="${state.view === id ? "on" : ""}" data-act="nav" data-id="${id}">${label}</button>`).join("");
  document.getElementById("app").innerHTML = `
    <header class="top">
      <div class="mark">Fablespace</div>
      <div class="story">${esc(project.title)}</div>
      <nav>${nav}</nav>
      <div class="actions">
        <button data-act="open-save">Save</button>
        <button class="primary" data-act="open-build">Build</button>
      </div>
    </header>
    <div class="shell">
      <aside>${renderList()}</aside>
      <main>
        <div class="work">${renderWork()}</div>
        ${state.notice ? `<div class="notice">${esc(state.notice)}</div>` : ""}
      </main>
    </div>
    ${renderModal()}`;
  paintMap();
  drawLinks();
  drawSkillLinks();
}

function paintMap() {
  const canvas = document.getElementById("map");
  if (!canvas) return;
  const map = mapOf();
  const ctx = canvas.getContext("2d");
  const scale = 7;
  const colors = {
    ".": "#6ea84a",
    "<": "#c4a36a",
    "=": "#b8956a",
    ">": "#c4a36a",
    R: "#c4473a",
    W: "#8d5a32",
    B: "#3e6ea8",
    S: "#8e98a3",
    "#": "#1c1916",
    "*": "#6e8f9a",
  };
  const rows = map.grid.split("\n").filter((row) => row.length);
  for (let y = 0; y < map.height; y += 1) {
    for (let x = 0; x < map.width; x += 1) {
      ctx.fillStyle = colors[rows[y]?.[x]] || "#6ea84a";
      ctx.fillRect(x * scale, y * scale, scale, scale);
    }
  }
  for (const entity of map.entities || []) {
    ctx.fillStyle = "#e2b15a";
    ctx.fillRect(entity.x * scale + 1, entity.y * scale + 1, scale - 2, scale - 2);
  }
  for (const npc of project.npcs) {
    if (!npc.place || npc.place.map !== map.id) continue;
    ctx.fillStyle = "#16324f";
    ctx.beginPath();
    ctx.arc(npc.place.x * scale + scale / 2, npc.place.y * scale + scale / 2, scale / 2 - 0.5, 0, Math.PI * 2);
    ctx.fill();
    ctx.strokeStyle = "#f7f3ea";
    ctx.lineWidth = 1;
    ctx.stroke();
  }
  ctx.strokeStyle = "#fffdf8";
  ctx.lineWidth = 1;
  ctx.strokeRect(state.tile.x * scale + 0.5, state.tile.y * scale + 0.5, scale - 1, scale - 1);
}

function curve(svg, x1, y1, x2, y2, failure, label) {
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  const mid = (x1 + x2) / 2;
  path.setAttribute("d", `M ${x1} ${y1} C ${mid} ${y1}, ${mid} ${y2}, ${x2} ${y2}`);
  path.setAttribute("fill", "none");
  path.setAttribute("stroke", failure ? "#a32622" : "#1d6b45");
  path.setAttribute("stroke-width", "2");
  if (failure) path.setAttribute("stroke-dasharray", "5 4");
  svg.appendChild(path);
  if (label) {
    const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
    text.setAttribute("x", mid);
    text.setAttribute("y", (y1 + y2) / 2 - 6);
    text.setAttribute("fill", failure ? "#a32622" : "#1d6b45");
    text.setAttribute("font-size", "12");
    text.setAttribute("font-family", "Segoe UI, sans-serif");
    text.textContent = label;
    svg.appendChild(text);
  }
}

function portPoint(board, el, side) {
  const box = board.getBoundingClientRect();
  const rect = el.getBoundingClientRect();
  const x = (side === "left" ? rect.left : rect.right) - box.left + board.scrollLeft;
  const y = rect.top + rect.height / 2 - box.top + board.scrollTop;
  return [x, y];
}

function drawLinks() {
  const board = document.getElementById("board");
  if (!board) return;
  const svg = board.querySelector("svg");
  svg.replaceChildren();
  svg.setAttribute("width", String(board.scrollWidth));
  svg.setAttribute("height", String(board.scrollHeight));
  board.querySelectorAll(".choice").forEach((choice) => {
    const target = board.querySelector(`.node[data-id="${CSS.escape(choice.dataset.goto)}"] .reqbox`);
    if (!target) return;
    const [x1, y1] = portPoint(board, choice, "right");
    const [x2, y2] = portPoint(board, target, "left");
    curve(svg, x1, y1, x2, y2, false, choice.childNodes[0].textContent.trim());
  });
  board.querySelectorAll(".failport").forEach((port) => {
    const node = board.querySelector(`.node[data-id="${CSS.escape(port.dataset.fail)}"]`);
    if (!node) return;
    const [x1, y1] = portPoint(board, port, "right");
    const [x2, y2] = portPoint(board, node, "left");
    curve(svg, x1, y1, x2, y2, true, "failure");
  });
}

function drawSkillLinks() {
  const board = document.getElementById("skillboard");
  if (!board) return;
  const svg = board.querySelector("svg");
  svg.replaceChildren();
  svg.setAttribute("width", String(board.scrollWidth));
  svg.setAttribute("height", String(board.scrollHeight));
  for (const skill of project.skills) {
    for (const need of skill.needNodes || []) {
      const from = board.querySelector(`.skillnode[data-skill="${CSS.escape(need)}"] header`);
      const to = board.querySelector(`.skillnode[data-skill="${CSS.escape(skill.id)}"] header`);
      if (!from || !to) continue;
      const [x1, y1] = portPoint(board, from, "right");
      const [x2, y2] = portPoint(board, to, "left");
      curve(svg, x1, y1, x2, y2, false, "");
    }
  }
}

function ensureBases(npc) {
  for (const stat of project.stats) {
    if (npc.bases[stat.id] == null) npc.bases[stat.id] = 0;
  }
}

function paintCell(x, y) {
  const map = mapOf();
  if (x < 0 || y < 0 || x >= map.width || y >= map.height) return;
  state.tile = { x, y };
  if (state.tool === "floor") setChar(map, x, y, ".");
  if (state.tool === "wall") setChar(map, x, y, "W");
  if (state.tool === "npc") {
    const npc = byId(project.npcs, state.paintNpc);
    if (npc) {
      npc.place = npc.place || { map: map.id, x, y, checks: [] };
      npc.place.map = map.id;
      npc.place.x = x;
      npc.place.y = y;
    }
  }
  if (state.tool === "entity") {
    map.entities = map.entities || [];
    const existing = map.entities.find((entity) => entity.x === x && entity.y === y);
    if (!existing) map.entities.push({ kind: "chest", name: "Chest", x, y });
  }
}

function setChar(map, x, y, ch) {
  const rows = map.grid.split("\n");
  const row = rows[y].split("");
  row[x] = ch;
  rows[y] = row.join("");
  map.grid = rows.join("\n");
}

function currentPassage() {
  return project.passages.find((passage) => passage.id === state.passage);
}

function addCheck(list, kind) {
  if (kind === "flag") list.push({ kind: "flag", flag: "flag", want: true });
  if (kind === "stat") list.push({ kind: "stat", stat: "strength", cmp: ">=", value: 1 });
  if (kind === "combat") list.push({ kind: "combat", win: true });
}

async function invoke(command, args) {
  const tauri = window.__TAURI__ || window.__TAURI_INTERNALS__;
  const fn = tauri?.core?.invoke || tauri?.invoke;
  if (!fn) return null;
  return fn(command, args);
}

async function saveProject() {
  const json = JSON.stringify(project, null, 2);
  const path = state.savePath.trim();
  try {
    const wrote = await invoke("save_project", { path, json });
    if (wrote === null) {
      const blob = new Blob([json], { type: "application/json" });
      const link = document.createElement("a");
      link.href = URL.createObjectURL(blob);
      link.download = path.split(/[\\/]/).pop() || "project.fable.json";
      link.click();
    }
    state.notice = "Saved the author project. The game was not built.";
  } catch (err) {
    state.notice = String(err);
  }
  state.modal = null;
  render();
}

async function buildProject() {
  const parent = state.buildParent.trim();
  const name = state.buildName.trim();
  if (!parent || !name) {
    state.notice = "Build needs a parent folder and a new folder name.";
    state.modal = "build";
    render();
    return;
  }
  try {
    const dest = await invoke("build_game", { parent, name, json: JSON.stringify(project) });
    if (dest === null) {
      state.notice = "Build runs in the desktop author.";
    } else {
      state.notice = "Built " + dest;
    }
    state.modal = null;
  } catch (err) {
    state.notice = String(err);
    state.modal = null;
  }
  render();
}

function onClick(event) {
  const target = event.target.closest("[data-act]");
  if (!target) {
    const node = event.target.closest(".node");
    if (node && !event.target.closest("header, .choice, .failport")) {
      state.passage = node.dataset.id;
      state.view = "passages";
      render();
    }
    const skill = event.target.closest(".skillnode");
    if (skill && event.target.tagName !== "HEADER") {
      state.skill = skill.dataset.skill;
      render();
    }
    return;
  }
  const act = target.dataset.act;
  const id = target.dataset.id;
  if (act === "nav") {
    state.view = id;
    state.folder = null;
    state.modal = null;
  } else if (act === "open-build") state.modal = "build";
  else if (act === "open-save") state.modal = "save";
  else if (act === "close-modal") state.modal = null;
  else if (act === "pick-passage") {
    state.passage = id.startsWith("place-") ? null : id;
    state.view = "passages";
  } else if (act === "pick-map") {
    state.map = id;
    state.view = "maps";
  } else if (act === "pick-npc") {
    state.npc = id;
    state.folder = null;
  } else if (act === "pick-item") {
    state.item = id;
    state.folder = null;
  } else if (act === "pick-folder") state.folder = id;
  else if (act === "pick-stat") state.stat = id;
  else if (act === "pick-skill") state.skill = id;
  else if (act === "tool") state.tool = id;
  else if (act === "add-passage") {
    const nid = "line" + project.passages.length;
    project.passages.push({ id: nid, kind: "passage", text: "...", x: 120, y: 420, checks: [], choices: [], mods: [] });
    state.passage = nid;
  } else if (act === "add-npc") {
    const nid = "npc" + (project.npcs.length + 1);
    const bases = {};
    project.stats.forEach((stat) => { bases[stat.id] = 0; });
    project.npcs.push({ id: nid, name: "New", folder: id, player: false, weapon: "", weaponAtk: 0, bases, items: [], skills: [], place: { map: "field", x: 2, y: 2, checks: [] } });
    state.npc = nid;
    state.folder = null;
  } else if (act === "add-item") {
    const nid = "item" + (project.items.length + 1);
    project.items.push({ id: nid, name: "New item", folder: id, equip: [], useMods: [] });
    state.item = nid;
    state.folder = null;
  } else if (act === "add-stat") {
    let n = project.stats.length + 1;
    let sid = "stat" + n;
    while (project.stats.some((stat) => stat.id === sid)) sid = "stat" + (++n);
    project.stats.push({ id: sid, name: "New stat" });
    project.npcs.forEach(ensureBases);
    state.stat = sid;
  } else if (act === "add-skill") {
    const nid = "skill" + (project.skills.length + 1);
    project.skills.push({ id: nid, name: "New skill", x: 40, y: 220, cost: [], needStats: [], needNodes: [] });
    state.skill = nid;
  } else if (act === "add-choice") {
    const passage = currentPassage();
    if (passage) {
      passage.choices = passage.choices || [];
      passage.choices.push({ label: "Next", goto: "end", sets: [], checks: [], mods: [] });
    }
  } else if (act === "add-node-check") {
    const passage = currentPassage();
    if (passage) {
      passage.checks = passage.checks || [];
      addCheck(passage.checks, target.dataset.check);
    }
  } else if (act === "add-choice-check") {
    const passage = currentPassage();
    const choice = passage?.choices?.[Number(target.dataset.index)];
    if (choice) {
      choice.checks = choice.checks || [];
      addCheck(choice.checks, target.dataset.check);
    }
  } else if (act === "add-mod") {
    const item = byId(project.items, state.item);
    if (!item) return;
    const list = target.dataset.id === "equip" ? (item.equip = item.equip || []) : (item.useMods = item.useMods || []);
    list.push({ stat: "hp", delta: 1 });
  } else if (act === "add-skill-cost") {
    const skill = byId(project.skills, state.skill);
    if (skill) (skill.cost = skill.cost || []).push({ stat: "strength", delta: 1 });
  } else if (act === "add-skill-stat") {
    const skill = byId(project.skills, state.skill);
    if (skill) (skill.needStats = skill.needStats || []).push({ kind: "stat", stat: "strength", cmp: ">=", value: 1 });
  } else if (act === "add-skill-node") {
    const skill = byId(project.skills, state.skill);
    const other = project.skills.find((item) => item.id !== skill?.id);
    if (skill && other && !(skill.needNodes || []).includes(other.id)) {
      skill.needNodes = skill.needNodes || [];
      skill.needNodes.push(other.id);
    }
  } else return;
  render();
}

function onChange(event) {
  const target = event.target;
  const field = target.dataset.field;
  const npc = byId(project.npcs, state.npc);
  const item = byId(project.items, state.item);
  if (target.dataset.base && npc) npc.bases[target.dataset.base] = Number(target.value);
  if (field === "passage-text") {
    const passage = currentPassage();
    if (passage) passage.text = target.value;
  } else if (field === "passage-slot") {
    const passage = currentPassage();
    if (passage) passage.slot = target.value || null;
  } else if (field === "npc-name" && npc) npc.name = target.value;
  else if (field === "npc-folder" && npc) npc.folder = target.value;
  else if (field === "npc-player" && npc) npc.player = target.checked;
  else if (field === "npc-weapon" && npc) npc.weapon = target.value;
  else if (field === "npc-weapon-atk" && npc) npc.weaponAtk = Number(target.value);
  else if (field === "place-map" && npc?.place) npc.place.map = target.value;
  else if (field === "place-x" && npc?.place) npc.place.x = Number(target.value);
  else if (field === "place-y" && npc?.place) npc.place.y = Number(target.value);
  else if (field === "item-name" && item) item.name = target.value;
  else if (field === "item-folder" && item) item.folder = target.value;
  else if (field === "folder-name") {
    const folder = byId(project.folders, state.folder);
    if (folder) folder.name = target.value;
  } else if (field === "stat-name") {
    const stat = byId(project.stats, state.stat);
    if (stat) stat.name = target.value;
  } else if (field === "skill-name") {
    const skill = byId(project.skills, state.skill);
    if (skill) skill.name = target.value;
  } else if (field === "build-parent") state.buildParent = target.value;
  else if (field === "build-name") state.buildName = target.value;
  else if (field === "save-path") state.savePath = target.value;
  else if (field === "paint-npc") state.paintNpc = target.value;
  else if (field === "art-owner") state.artOwner = target.value;
  else if (field === "art-role") state.artRole = target.value;
  else if (target.dataset.choice != null) {
    const passage = currentPassage();
    const choice = passage?.choices?.[Number(target.dataset.choice)];
    if (!choice) return;
    if (target.dataset.key === "sets") choice.sets = target.value.split(/\s+/).filter(Boolean);
    else if (target.dataset.key === "fail") choice.fail = target.value || null;
    else choice[target.dataset.key] = target.value;
  } else if (target.dataset.mod) {
    const list = target.dataset.mod === "equip" ? item?.equip : item?.useMods;
    const mod = list?.[Number(target.dataset.index)];
    if (!mod) return;
    mod[target.dataset.key] = target.dataset.key === "delta" ? Number(target.value) : target.value;
  }
}

function onSubmit(event) {
  const form = event.target.closest("form");
  if (!form) return;
  event.preventDefault();
  if (form.dataset.act === "save-submit") saveProject();
  if (form.dataset.act === "build-submit") buildProject();
}

function readImage(file, done) {
  const reader = new FileReader();
  reader.onload = () => {
    const image = new Image();
    image.onload = () => done(reader.result, image.width, image.height);
    image.src = reader.result;
  };
  reader.readAsDataURL(file);
}

function onFile(event) {
  const file = event.target.files && event.target.files[0];
  if (!file) return;
  const kind = event.target.dataset.import;
  readImage(file, (data, width, height) => {
    project.art = project.art || { tiles: null, sprites: [] };
    if (kind === "tiles") project.art.tiles = data;
    if (kind === "sprite") {
      const frames = height === 16 && width % 16 === 0 ? width / 16 : 1;
      project.art.sprites = project.art.sprites.filter((sprite) => !(sprite.owner === state.artOwner && sprite.role === state.artRole));
      project.art.sprites.push({ owner: state.artOwner, role: state.artRole, frames, data });
    }
    state.notice = kind === "tiles" ? "Imported tiles." : `Imported ${framesLabel(width, height)}`;
    render();
  });
}

function framesLabel(width, height) {
  if (height === 16 && width % 16 === 0) return `${width / 16} frames`;
  return "a sprite";
}

let drag = null;
function onPointerDown(event) {
  const header = event.target.closest(".node header, .skillnode header");
  if (!header) return;
  const el = header.parentElement;
  drag = {
    el,
    id: el.dataset.id || el.dataset.skill,
    skill: Boolean(el.dataset.skill),
    npc: el.dataset.npc,
    x: event.clientX,
    y: event.clientY,
    left: parseInt(el.style.left, 10) || 0,
    top: parseInt(el.style.top, 10) || 0,
  };
}

function onPointerMove(event) {
  if (!drag) return;
  drag.el.style.left = drag.left + event.clientX - drag.x + "px";
  drag.el.style.top = drag.top + event.clientY - drag.y + "px";
  drawLinks();
  drawSkillLinks();
}

function onPointerUp() {
  if (!drag) return;
  const x = parseInt(drag.el.style.left, 10) || 0;
  const y = parseInt(drag.el.style.top, 10) || 0;
  if (drag.skill) {
    const skill = byId(project.skills, drag.id);
    if (skill) {
      skill.x = x;
      skill.y = y;
    }
  } else if (drag.npc) {
    const npc = byId(project.npcs, drag.npc);
    if (npc?.place) {
      npc.place.ux = x;
      npc.place.uy = y;
    }
  } else {
    const passage = project.passages.find((item) => item.id === drag.id);
    if (passage) {
      passage.x = x;
      passage.y = y;
    }
  }
  drag = null;
}

function onMapPointer(event) {
  const canvas = event.target.closest("#map");
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const x = Math.floor((event.clientX - rect.left) * (canvas.width / rect.width) / 7);
  const y = Math.floor((event.clientY - rect.top) * (canvas.height / rect.height) / 7);
  paintCell(x, y);
  render();
}

applyHash();
const root = document.getElementById("app");
root.addEventListener("click", onClick);
root.addEventListener("change", onChange);
root.addEventListener("submit", onSubmit);
root.addEventListener("pointerdown", (event) => {
  onPointerDown(event);
  onMapPointer(event);
});
window.addEventListener("pointermove", onPointerMove);
window.addEventListener("pointerup", onPointerUp);
root.addEventListener("change", (event) => {
  if (event.target.dataset.import) onFile(event);
});
render();
