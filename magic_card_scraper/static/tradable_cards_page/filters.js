// Initialize Tablesort
new Tablesort(document.getElementById('card-table'), {
    descending: true
});

// Utility function to escape HTML special characters
function escapeHTML(str) {
    return str.replace(/[&<>"']/g, function (match) {
        const escapeMap = {
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            '"': '&quot;',
            "'": '&#39;'
        };
        return escapeMap[match];
    });
}

// Function to populate filter dropdowns dynamically
function populateFilters() {
    const rows = document.querySelectorAll('#card-table tbody tr');
    const raritySet = new Set();
    const colorSet = new Set();

    // Collect unique values
    rows.forEach(row => {
        const rarity = row.querySelector('td:nth-child(8)').textContent.trim();
        const color = row.querySelector('td:nth-child(7)').textContent.trim();

        raritySet.add(rarity);
        colorSet.add(color);
    });

    // Populate rarity filter
    const rarityFilter = document.getElementById('rarityFilter');
    rarityFilter.innerHTML = '<option value="all">All</option>';
    Array.from(raritySet).sort().forEach(rarity => {
        rarityFilter.innerHTML += `<option value="${escapeHTML(rarity)}">${escapeHTML(rarity)}</option>`;
    });

    // Populate color filter
    const colorFilter = document.getElementById('colorFilter');
    colorFilter.innerHTML = '<option value="all">All</option>';
    Array.from(colorSet).sort().forEach(color => {
        colorFilter.innerHTML += `<option value='${escapeHTML(color)}'>${escapeHTML(color)}</option>`;
    });
}

// Filter function
function applyFilters() {
    const rarityFilter = document.getElementById('rarityFilter').value;
    const colorFilter = document.getElementById('colorFilter').value;
    const showValueTrades = document.getElementById('valueTradeFilter').checked;
    const minPriceFilter = parseFloat(document.getElementById('minPriceFilter').value) || 0;
    const minDiffFilter = parseFloat(document.getElementById('minDiffFilter').value) || 0;
    const rows = document.querySelectorAll('#card-table tbody tr');

    rows.forEach(row => {
        const rarity = row.querySelector('td:nth-child(8)').textContent.trim();
        const color = row.querySelector('td:nth-child(7)').textContent.trim();
        const tradeInPrice = parseFloat(row.querySelector('td:nth-child(3)').dataset.sort);
        const percentualDifference = parseFloat(row.querySelector('td:nth-child(9)').dataset.sort);
        const isValueTrade = row.dataset.valueTrade === 'true';

        const rarityMatch = rarityFilter === 'all' || rarity === rarityFilter;
        const colorMatch = colorFilter === 'all' || color === colorFilter;
        const minPriceMatch = tradeInPrice >= minPriceFilter;
        const minDiffMatch = percentualDifference >= minDiffFilter;

        if (rarityMatch && colorMatch && minPriceMatch && minDiffMatch && (!showValueTrades || isValueTrade)) {
            row.classList.remove('hidden');
        } else {
            row.classList.add('hidden');
        }
    });
}

// Reset filters
function resetFilters() {
    document.getElementById('rarityFilter').value = 'all';
    document.getElementById('colorFilter').value = 'all';
    document.getElementById('minPriceFilter').value = '';
    document.getElementById('valueTradeFilter').checked = false;
    document.getElementById('minDiffFilter').value = '';
    const rows = document.querySelectorAll('#card-table tbody tr');
    rows.forEach(row => row.classList.remove('hidden'));
}

// Filter value trades
function filterValueTrades() {
    document.getElementById('valueTradeFilter').checked = true;
    applyFilters();
}

// Initialize filters
populateFilters();

// Add event listeners to filters
document.getElementById('rarityFilter').addEventListener('change', applyFilters);
document.getElementById('colorFilter').addEventListener('change', applyFilters);
document.getElementById('valueTradeFilter').addEventListener('change', applyFilters);
document.getElementById('minPriceFilter').addEventListener('input', applyFilters);
document.getElementById('minDiffFilter').addEventListener('input', applyFilters);