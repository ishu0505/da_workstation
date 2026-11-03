// Pure layout geometry. No GNOME imports, so tests can run it with plain gjs.

export const GAP = 12;

/**
 * Stage: one big window, the others as small cards in a column on the right. Every
 * window keeps the big size (desktop proportions); a card is that window drawn at
 * `scale`. Returns {big, cards, scale}; `cards` has count - 1 rects (where each card
 * is drawn). Throws when the area is too small.
 */
export function stage(area, count, maxColumn = 240) {
    // On small screens the card column shrinks first (at most a fifth of the width).
    const column = Math.min(maxColumn, Math.floor(area.width * 0.2));
    if (count < 2)
        throw new Error('Open at least two normal windows on this monitor and workspace');
    const big = {x: area.x + GAP, y: area.y + GAP,
        width: area.width - column - GAP * 3, height: area.height - GAP * 2};
    if (big.width < 640 || big.height < 240)
        throw new Error('The screen is too small to tile. Close the sidebar or use a larger screen.');
    const n = count - 1;
    // Fit the column's width, and all cards into the column's height.
    const scale = Math.min(column / big.width, (big.height - GAP * (n - 1)) / (n * big.height));
    const width = Math.floor(big.width * scale);
    const height = Math.floor(big.height * scale);
    const x = big.x + big.width + GAP + Math.floor((column - width) / 2);
    const cards = Array.from({length: n}, (_, i) =>
        ({x, y: big.y + i * (height + GAP), width, height}));
    return {big, cards, scale};
}

/** The window whose centre is nearest in a direction; distance across the direction costs double. */
export function nearest(from, candidates, direction) {
    const centre = r => ({x: r.x + r.width / 2, y: r.y + r.height / 2});
    const a = centre(from);
    let best = null;
    let bestScore = Infinity;
    for (const candidate of candidates) {
        const b = centre(candidate.rect);
        const dx = b.x - a.x;
        const dy = b.y - a.y;
        const [along, across] = {
            left: [-dx, dy], right: [dx, dy], up: [-dy, dx], down: [dy, dx],
        }[direction] ?? [NaN, NaN];
        if (!(along > 1))
            continue;
        const score = along + 2 * Math.abs(across);
        if (score < bestScore) {
            bestScore = score;
            best = candidate;
        }
    }
    return best;
}

/** Cut the left edge of the work area so tiled windows do not cover the sidebar. */
export function besideSidebar(area, sidebar) {
    if (!sidebar)
        return area;
    // Docked along the bottom (wider than half the area): keep the space above it.
    if (sidebar.width > area.width / 2) {
        if (sidebar.y <= area.y || sidebar.y >= area.y + area.height)
            return area;
        return {x: area.x, y: area.y, width: area.width, height: sidebar.y - area.y};
    }
    const right = sidebar.x + sidebar.width;
    if (right <= area.x || right >= area.x + area.width)
        return area;
    return {x: right, y: area.y, width: area.x + area.width - right, height: area.height};
}
