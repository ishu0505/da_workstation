// Run: gjs -m experiments/gnome-focus/layout_test.js
import {GAP, besideSidebar, nearest, stage} from '../../gnome-extension/layout.js';

const assert = (ok, message) => { if (!ok) throw new Error(message); };
const area = {x: 0, y: 32, width: 1920, height: 1168};
const usable = besideSidebar(area, {x: 0, y: 32, width: 380, height: 1168});
const inside = (r, a) => r.x >= a.x && r.y >= a.y && r.x + r.width <= a.x + a.width &&
    r.y + r.height <= a.y + a.height;
const apart = (a, b) => a.x + a.width <= b.x || b.x + b.width <= a.x ||
    a.y + a.height <= b.y || b.y + b.height <= a.y;
for (const count of [2, 3, 6, 9]) {
    const {big, cards, scale} = stage(usable, count);
    assert(cards.length === count - 1, `cards ${count}`);
    assert(big.width >= 1200 && inside(big, usable), `big window is desktop-size ${big.width}`);
    for (const [i, c] of cards.entries()) {
        assert(inside(c, usable) && apart(c, big), `card ${i} of ${count} beside the big window`);
        assert(Math.abs(c.width / c.height - big.width / big.height) < 0.02, 'cards keep proportions');
        assert(Math.abs(c.width - big.width * scale) < 1, 'card is the window at scale');
        for (const d of cards.slice(i + 1))
            assert(apart(c, d), `cards do not overlap ${count}`);
    }
}
let threw = false;
try { stage(area, 1); } catch { threw = true; }
assert(threw, 'one window is refused');
threw = false;
try { stage({x: 0, y: 0, width: 800, height: 400}, 3); } catch { threw = true; }
assert(threw, 'too small is refused');
const from = {x: 0, y: 0, width: 100, height: 100};
const right = {rect: {x: 200, y: 0, width: 100, height: 100}};
const below = {rect: {x: 0, y: 200, width: 100, height: 100}};
assert(nearest(from, [right, below], 'right') === right, 'right');
assert(nearest(from, [right, below], 'down') === below, 'down');
assert(nearest(from, [right, below], 'left') === null, 'nothing left');
const cut = besideSidebar(area, {x: 0, y: 32, width: 360, height: 1168});
assert(cut.x === 360 && cut.width === 1560, 'sidebar cut');
assert(besideSidebar(area, null) === area, 'no sidebar');
const above = besideSidebar(area, {x: 0, y: 900, width: 1920, height: 300});
assert(above.y === 32 && above.height === 868 && above.width === 1920, 'bottom bar keeps the space above');
assert(stage(above, 4).big.height <= 868, 'stage fits above the bottom bar');
assert(GAP === 12, 'gap');
print('PASS: layout geometry, direction and sidebar tests');
