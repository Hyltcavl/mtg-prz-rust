// Initialize Tablesort
new Tablesort(document.getElementById('card-table'), {
    descending: true
});

// Function to populate filter dropdowns dynamically
function populateFilters() {
    const rows = document.querySelectorAll('#card-table tbody tr');
    const vendorSet = new Set();
    
    // Collect unique values
    rows.forEach(row => {
        const vendor = row.querySelector('td:nth-child(6)').textContent.trim();                
        vendorSet.add(vendor);
    });

    // Populate vendor filter
    const vendorFilter = document.getElementById('vendorFilter');
    vendorFilter.innerHTML = '<option value="all">All</option>';
    Array.from(vendorSet).sort().forEach(vendor => {
        vendorFilter.innerHTML += `<option value="${vendor}">${vendor}</option>`;
    });
}

// Filter function
function applyFilters() {
    const vendorFilter = document.getElementById('vendorFilter').value;
    const rows = document.querySelectorAll('#card-table tbody tr');

    rows.forEach(row => {
        const vendor = row.querySelector('td:nth-child(6)').textContent.trim();
        
        const vendorMatch = vendorFilter === 'all' || vendor === vendorFilter;

        if (vendorMatch) {
            row.classList.remove('hidden');
        } else {
            row.classList.add('hidden');
        }
    });
}

// Reset filters
function resetFilters() {
    document.getElementById('vendorFilter').value = 'all';
    const rows = document.querySelectorAll('#card-table tbody tr');
    rows.forEach(row => row.classList.remove('hidden'));
}

// Initialize filters
populateFilters();

// Add event listeners to filters
document.getElementById('vendorFilter').addEventListener('change', applyFilters);