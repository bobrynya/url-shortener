/**
 * URL Shortener - JavaScript Library
 * Fully refactored with consistent naming conventions
 */

// ============================================
// AUTH MODULE
// ============================================
let Auth = {
    /**
     * Get auth token from cookie
     */
    getToken() {
        const cookies = document.cookie.split(';');
        for (let cookie of cookies) {
            const [name, value] = cookie.trim().split('=');
            if (name === 'auth_token') {
                return value;
            }
        }
        return null;
    },

    /**
     * Remove auth token cookie
     */
    logout() {
        document.cookie = 'auth_token=; path=/; max-age=0';
        this.redirectToLogin();
    },

    redirectToLogin() {
        window.location.href = '/dashboard/login';
    }
};

// ============================================
// UTILITY FUNCTIONS
// ============================================
const Utils = {
    /**
     * Convert API response from snake_case to camelCase
     * @param {object} obj - Object with snake_case keys
     * @returns {object} Object with camelCase keys
     */
    toCamelCase(obj) {
        if (obj === null || typeof obj !== 'object') return obj;

        if (Array.isArray(obj)) {
            return obj.map(item => this.toCamelCase(item));
        }

        const camelObj = {};
        for (const key in obj) {
            if (obj.hasOwnProperty(key)) {
                const camelKey = key.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
                camelObj[camelKey] = this.toCamelCase(obj[key]);
            }
        }
        return camelObj;
    },

    /**
     * Format date string to human-readable format
     * @param {string} dateString - ISO 8601 / RFC3339 date
     * @returns {string}
     */
    formatDate(dateString) {
        const date = new Date(dateString);
        const now = new Date();
        const diff = now - date;
        const days = Math.floor(diff / (1000 * 60 * 60 * 24));

        if (days === 0) return 'today';
        if (days === 1) return 'yesterday';
        if (days < 7) return `${days} days ago`;

        return date.toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    },

    /**
     * Format date and time to full string
     * @param {string} dateString - ISO date
     * @returns {string}
     */
    formatDateTime(dateString) {
        const date = new Date(dateString);
        return date.toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'long',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit'
        });
    },

    /**
     * Copy text to clipboard with button feedback
     * @param {string} text - Text to copy
     * @param {HTMLElement} button - Button that triggered the action
     */
    async copyToClipboard(text, button) {
        try {
            await navigator.clipboard.writeText(text);

            const originalText = button.textContent;
            button.textContent = '✓ Copied';
            button.style.background = '#10b981';

            setTimeout(() => {
                button.textContent = originalText;
                button.style.background = '';
            }, 2000);
        } catch (err) {
            console.error('Failed to copy:', err);
            alert('Failed to copy to clipboard');
        }
    },

    /**
     * Show error message in container
     * @param {string} message - Error message
     * @param {HTMLElement} container - Container element
     */
    showError(message, container) {
        container.innerHTML = `<div class="error">${message}</div>`;
    },

    /**
     * Show loading state
     * @param {HTMLElement} container - Container element
     */
    showLoading(container) {
        container.innerHTML = `<div class="loading">Loading...</div>`;
    },

    /**
     * Show empty state
     * @param {string} message - Empty state message
     * @param {HTMLElement} container - Container element
     */
    showEmpty(message, container) {
        container.innerHTML = `<div class="empty-state"><p>${message}</p></div>`;
    }
};

// ============================================
// API MODULE
// ============================================
const API = {
    /**
     * Base API request
     * @param {string} endpoint - API endpoint
     * @param {object} options - fetch options
     * @returns {Promise}
     */
    async request(endpoint, options = {}) {
        const token = Auth.getToken();

        const response = await fetch(endpoint, {
            headers: {
                'Content-Type': 'application/json',
                ...(token ? { 'Authorization': `Bearer ${token}` } : {}),
                ...options.headers
            },
            ...options
        });

        // Handle 401 - unauthorized
        if (response.status === 401) {
            alert('Session expired. Please log in again.');
            Auth.logout();
            return null;
        }

        // Handle other errors
        if (!response.ok) {
            const error = new Error(`HTTP error! status: ${response.status}`);
            console.error('API Error:', error);
            throw error;
        }

        const data = await response.json();
        // Convert snake_case to camelCase
        return Utils.toCamelCase(data);
    },

    /**
     * Get list of domains
     * @returns {Promise}
     */
    async getDomains() {
        return this.request('/api/domains');
    },

    /**
     * Get links with pagination and filters
     * @param {object} params - {page, pageSize, from, to, domain}
     * @returns {Promise}
     */
    async getLinks(params = {}) {
        const queryParams = new URLSearchParams();

        if (params.page) queryParams.append('page', params.page);
        if (params.pageSize) queryParams.append('page_size', params.pageSize);
        if (params.from) queryParams.append('from', params.from);
        if (params.to) queryParams.append('to', params.to);
        if (params.domain) queryParams.append('domain', params.domain);

        const query = queryParams.toString();
        return this.request(`/api/stats${query ? '?' + query : ''}`);
    },

    /**
     * Create short links
     * @param {Array} urls - [{url, customCode?, domain?}]
     * @returns {Promise}
     */
    async createLinks(urls) {
        // Convert camelCase to snake_case for API
        const apiUrls = urls.map(item => ({
            url: item.url,
            ...(item.customCode ? { custom_code: item.customCode } : {}),
            ...(item.domain ? { domain: item.domain } : {})
        }));

        return this.request('/api/shorten', {
            method: 'POST',
            body: JSON.stringify({ urls: apiUrls })
        });
    },

    /**
     * Get link statistics
     * @param {string} code - Short code
     * @param {object} params - {page, pageSize, from, to}
     * @returns {Promise}
     */
    async getLinkStats(code, params = {}) {
        const queryParams = new URLSearchParams();

        if (params.page) queryParams.append('page', params.page);
        if (params.pageSize) queryParams.append('page_size', params.pageSize);
        if (params.from) queryParams.append('from', params.from);
        if (params.to) queryParams.append('to', params.to);

        const query = queryParams.toString();
        return this.request(`/api/stats/${code}${query ? '?' + query : ''}`);
    }
};

// ============================================
// DASHBOARD MODULE
// ============================================
const Dashboard = {
    domains: [],
    linkFields: [],

    /**
     * Load available domains from API
     */
    async loadDomains() {
        try {
            const data = await API.getDomains();
            this.domains = data.items || [];
        } catch (error) {
            console.error('Failed to load domains:', error);
            this.domains = [];
        }
    },

    /**
     * Load recent links
     */
    async loadRecentLinks() {
        const container = document.getElementById('recentLinks');
        Utils.showLoading(container);

        try {
            const data = await API.getLinks({ page: 1, pageSize: 20 });

            if (!data || !data.items || data.items.length === 0) {
                Utils.showEmpty('No links created yet', container);
                return;
            }

            // Render table
            container.innerHTML = `
                <table>
                    <thead>
                        <tr>
                            <th>Code</th>
                            <th>Domain</th>
                            <th>Original URL</th>
                            <th>Clicks</th>
                            <th>Created</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${data.items.map(link => `
                            <tr>
                                <td>
                                    <a href="https://${link.domain}/${link.code}" target="_blank">
                                        ${link.code}
                                    </a>
                                </td>
                                <td><code>${link.domain}</code></td>
                                <td style="max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${link.longUrl}">
                                    ${link.longUrl}
                                </td>
                                <td>${link.total || 0}</td>
                                <td>${Utils.formatDate(link.createdAt)}</td>
                                <td>
                                    <div class="actions">
                                        <a href="/dashboard/stats/${link.code}" class="btn btn-sm">📊</a>
                                        <button onclick="Utils.copyToClipboard('https://${link.domain}/${link.code}', this)" class="btn btn-sm btn-secondary">📋</button>
                                    </div>
                                </td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            `;
        } catch (error) {
            Utils.showError(error.message, container);
        }
    },

    /**
     * Add new link input field
     */
    addLinkField() {
        const container = document.getElementById('linkFieldsContainer');
        const fieldId = Date.now();
        this.linkFields.push(fieldId);

        const domainOptions = this.domains
            .filter(d => d.isActive)
            .map(d => `<option value="${d.domain}" ${d.isDefault ? 'selected' : ''}>${d.domain}</option>`)
            .join('');

        const fieldHTML = `
            <div class="link-field" id="field-${fieldId}">
                <div class="link-field-header">
                    <h3>Link #${this.linkFields.length}</h3>
                    ${this.linkFields.length > 1 ? `
                        <button type="button" onclick="Dashboard.removeLinkField(${fieldId})" class="btn btn-sm btn-danger">✕</button>
                    ` : ''}
                </div>
                <div class="link-field-content">
                    <div class="form-group">
                        <label>URL</label>
                        <input type="url" class="link-url" data-field-id="${fieldId}" required placeholder="https://example.com/very-long-url">
                    </div>
                    <div class="form-row">
                        <div class="form-group">
                            <label>Domain</label>
                            <select class="link-domain" data-field-id="${fieldId}">
                                ${domainOptions}
                            </select>
                        </div>
                        <div class="form-group">
                            <label>Custom Code</label>
                            <input type="text" class="link-custom-code" data-field-id="${fieldId}" pattern="[a-zA-Z0-9_\\-]+" placeholder="my-custom-link">
                            <small>Optional, leave blank for auto-generated</small>
                        </div>
                    </div>
                </div>
            </div>
        `;

        container.insertAdjacentHTML('beforeend', fieldHTML);
    },

    /**
     * Remove link field by ID
     */
    removeLinkField(fieldId) {
        const field = document.getElementById(`field-${fieldId}`);
        if (field) {
            field.remove();
            this.linkFields = this.linkFields.filter(id => id !== fieldId);
            this.updateFieldNumbers();
        }
    },

    /**
     * Update field numbers after removal
     */
    updateFieldNumbers() {
        const fields = document.querySelectorAll('.link-field');
        fields.forEach((field, index) => {
            const header = field.querySelector('.link-field-header h3');
            if (header) {
                header.textContent = `Link #${index + 1}`;
            }
        });
    },

    /**
     * Collect form data from all fields
     */
    collectFormData() {
        const urls = [];
        const urlInputs = document.querySelectorAll('.link-url');

        urlInputs.forEach(input => {
            const fieldId = input.dataset.fieldId;
            const url = input.value.trim();

            if (url) {
                const domainSelect = document.querySelector(`.link-domain[data-field-id="${fieldId}"]`);
                const customCodeInput = document.querySelector(`.link-custom-code[data-field-id="${fieldId}"]`);

                const linkData = { url };

                // Add domain if not default
                const domain = domainSelect?.value;
                const defaultDomain = this.domains.find(d => d.isDefault)?.domain;
                if (domain && domain !== defaultDomain) {
                    linkData.domain = domain;
                }

                // Add custom code if provided
                const customCode = customCodeInput?.value.trim();
                if (customCode) {
                    linkData.customCode = customCode;
                }

                urls.push(linkData);
            }
        });

        return urls;
    },

    /**
     * Display creation results
     */
    displayResults(response) {
        const container = document.getElementById('createResult');
        const { summary, items } = response;

        let html = `
            <div class="result">
                <h2>${summary.successful > 0 ? '✅ Links Created' : '❌ Failed'} (${summary.successful}/${summary.total})</h2>
                <div class="results-list">
        `;

        items.forEach(item => {
            if (item.error) {
                html += `
                    <div class="result-item result-error">
                        <div class="result-url-display">
                            <span class="result-icon">❌</span>
                            <span class="result-long-url">${item.longUrl}</span>
                        </div>
                        <div class="result-message error">
                            ${item.error.message}
                            ${item.error.details ? `<br><small>${JSON.stringify(item.error.details)}</small>` : ''}
                        </div>
                    </div>
                `;
            } else {
                html += `
                    <div class="result-item result-success">
                        <div class="result-url-display">
                            <span class="result-icon">✅</span>
                            <span class="result-long-url">${item.longUrl}</span>
                        </div>
                        <div class="result-short">
                            <input type="text" value="${item.shortUrl}" readonly>
                            <button onclick="Utils.copyToClipboard('${item.shortUrl}', this)" class="btn btn-sm">📋</button>
                            <a href="/dashboard/stats/${item.code}" class="btn btn-sm btn-secondary">📊</a>
                        </div>
                    </div>
                `;
            }
        });

        html += `
                </div>
            </div>
        `;

        container.innerHTML = html;
    },

    /**
     * Handle form submission
     */
    async handleCreateLinks(event) {
        event.preventDefault();

        const submitBtn = event.target.querySelector('button[type="submit"]');
        const originalBtn = submitBtn.textContent;
        submitBtn.disabled = true;
        submitBtn.textContent = 'Creating...';

        const urls = this.collectFormData();

        if (urls.length === 0) {
            alert('Please enter at least one URL');
            submitBtn.disabled = false;
            submitBtn.textContent = originalBtn;
            return;
        }

        try {
            const result = await API.createLinks(urls);
            this.displayResults(result);

            // Reload recent links if at least one was successful
            if (result.summary.successful > 0) {
                await this.loadRecentLinks();
            }
        } catch (error) {
            alert(`Error: ${error.message}`);
        } finally {
            submitBtn.disabled = false;
            submitBtn.textContent = originalBtn;
        }
    },

    /**
     * Initialize Dashboard
     */
    async init() {
        await this.loadDomains();
        await this.loadRecentLinks();

        // Add first link field
        this.addLinkField();

        // Add field button
        const addFieldBtn = document.getElementById('addLinkFieldBtn');
        if (addFieldBtn) {
            addFieldBtn.addEventListener('click', () => this.addLinkField());
        }

        // Form submission
        const createForm = document.getElementById('createLinksForm');
        if (createForm) {
            createForm.addEventListener('submit', (e) => this.handleCreateLinks(e));
        }
    }
};

// ============================================
// LINKS MODULE
// ============================================
const Links = {
    state: {
        currentPage: 1,
        totalPages: 1,
        pageSize: 25,
        fromDate: '',
        toDate: '',
        selectedDomain: '',
    },
    domains: [],

    /**
     * Load available domains
     */
    async loadDomains() {
        try {
            const data = await API.getDomains();
            this.domains = data.items || [];
            this.populateDomainFilter();
        } catch (error) {
            console.error('Failed to load domains:', error);
        }
    },

    /**
     * Populate domain filter dropdown
     */
    populateDomainFilter() {
        const domainSelect = document.getElementById('domainFilter');
        if (!domainSelect) return;

        domainSelect.innerHTML = `<option value="">All Domains</option>`;
        domainSelect.innerHTML += this.domains
            .filter(d => d.isActive)
            .map(d => `<option value="${d.domain}">${d.domain}</option>`)
            .join('');
    },

    /**
     * Load links with current filters
     */
    async loadLinks() {
        const container = document.getElementById('linksTable');
        Utils.showLoading(container);

        try {
            const params = {
                page: this.state.currentPage,
                pageSize: this.state.pageSize
            };

            if (this.state.fromDate) params.from = this.state.fromDate;
            if (this.state.toDate) params.to = this.state.toDate;
            if (this.state.selectedDomain) params.domain = this.state.selectedDomain;

            const data = await API.getLinks(params);

            if (!data || !data.items || data.items.length === 0) {
                Utils.showEmpty('No links found', container);
                return;
            }

            this.state.totalPages = data.pagination?.totalPages || 1;

            // Render table
            container.innerHTML = `
                <table>
                    <thead>
                        <tr>
                            <th>Code</th>
                            <th>Domain</th>
                            <th>Original URL</th>
                            <th>Clicks</th>
                            <th>Created</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${data.items.map(link => `
                            <tr>
                                <td>
                                    <a href="https://${link.domain}/${link.code}" target="_blank">
                                        <code>${link.code}</code>
                                    </a>
                                </td>
                                <td><code>${link.domain}</code></td>
                                <td style="max-width: 400px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${link.longUrl}">
                                    ${link.longUrl}
                                </td>
                                <td>${link.total || 0}</td>
                                <td>${Utils.formatDate(link.createdAt)}</td>
                                <td>
                                    <div class="actions">
                                        <a href="/dashboard/stats/${link.code}" class="btn btn-sm">📊</a>
                                        <button onclick="Links.copyLink('${link.domain}', '${link.code}', this)" class="btn btn-sm btn-secondary">📋</button>
                                    </div>
                                </td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
                <div class="pagination-info">
                    Showing ${data.items.length} of ${data.pagination?.totalItems || 0} links
                </div>
            `;

            this.renderPagination();
        } catch (error) {
            Utils.showError(error.message, container);
        }
    },

    /**
     * Copy link to clipboard
     */
    async copyLink(domain, code, button) {
        const url = `https://${domain}/${code}`;
        await Utils.copyToClipboard(url, button);
    },

    /**
     * Render pagination controls
     */
    renderPagination() {
        const container = document.getElementById('pagination');
        if (!container) return;

        const { currentPage, totalPages } = this.state;
        let html = '<div class="pagination">';

        // Previous button
        html += `<button ${currentPage === 1 ? 'disabled' : ''} onclick="Links.goToPage(${currentPage - 1})">← Previous</button>`;

        // Page numbers
        for (let i = 1; i <= totalPages; i++) {
            if (i === 1 || i === totalPages || (i >= currentPage - 2 && i <= currentPage + 2)) {
                html += `<button class="${i === currentPage ? 'active' : ''}" onclick="Links.goToPage(${i})">${i}</button>`;
            } else if (i === currentPage - 3 || i === currentPage + 3) {
                html += `<button disabled>...</button>`;
            }
        }

        // Next button
        html += `<button ${currentPage === totalPages ? 'disabled' : ''} onclick="Links.goToPage(${currentPage + 1})">Next →</button>`;

        html += '</div>';
        container.innerHTML = html;
    },

    /**
     * Go to specific page
     */
    async goToPage(page) {
        this.state.currentPage = page;
        await this.loadLinks();
    },

    /**
     * Apply filters
     */
    async applyFilters() {
        const pageSize = document.getElementById('pageSizeSelect')?.value || '25';
        const fromDate = document.getElementById('fromDate')?.value || '';
        const toDate = document.getElementById('toDate')?.value || '';
        const domain = document.getElementById('domainFilter')?.value || '';

        this.state.pageSize = parseInt(pageSize);
        this.state.fromDate = fromDate ? new Date(fromDate).toISOString() : '';
        this.state.toDate = toDate ? new Date(toDate).toISOString() : '';
        this.state.selectedDomain = domain;
        this.state.currentPage = 1;

        await this.loadLinks();
    },

    /**
     * Reset all filters
     */
    async resetFilters() {
        document.getElementById('fromDate').value = '';
        document.getElementById('toDate').value = '';
        document.getElementById('domainFilter').value = '';
        await this.applyFilters();
    },

    /**
     * Initialize Links page
     */
    async init() {
        await this.loadDomains();
        await this.loadLinks();

        const pageSizeSelect = document.getElementById('pageSizeSelect');
        const fromDate = document.getElementById('fromDate');
        const toDate = document.getElementById('toDate');
        const domainFilter = document.getElementById('domainFilter');
        const applyBtn = document.getElementById('applyFiltersBtn');
        const resetBtn = document.getElementById('resetFiltersBtn');

        if (pageSizeSelect) pageSizeSelect.addEventListener('change', () => this.applyFilters());
        if (fromDate) fromDate.addEventListener('change', () => this.applyFilters());
        if (toDate) toDate.addEventListener('change', () => this.applyFilters());
        if (domainFilter) domainFilter.addEventListener('change', () => this.applyFilters());
        if (applyBtn) applyBtn.addEventListener('click', () => this.applyFilters());
        if (resetBtn) resetBtn.addEventListener('click', () => this.resetFilters());
    }
};

// ============================================
// STATS MODULE
// ============================================
const Stats = {
    code: null,
    chart: null,
    allClicksData: null,
    state: {
        currentPage: 1,
        totalPages: 1,
        pageSize: 25,
        fromDate: '',
        toDate: '',
        currentPeriod: 'all',
    },

    /**
     * Set quick filter period
     */
    async setQuickFilter(period) {
        this.state.currentPeriod = period;

        // Update button states
        document.querySelectorAll('.quick-filter-btn').forEach(btn => btn.classList.remove('active'));
        document.querySelector(`[data-period="${period}"]`).classList.add('active');

        // Hide custom period form
        document.getElementById('customPeriodForm').style.display = 'none';

        const now = new Date();
        let from = null;

        switch (period) {
            case 'today':
                from = new Date(now.getFullYear(), now.getMonth(), now.getDate());
                break;
            case 'week':
                from = new Date(now);
                from.setDate(from.getDate() - 7);
                break;
            case 'month':
                from = new Date(now);
                from.setMonth(from.getMonth() - 1);
                break;
            case 'all':
                from = null;
                break;
        }

        this.state.fromDate = from ? from.toISOString() : '';
        this.state.toDate = now.toISOString();
        this.state.currentPage = 1;

        await this.loadLinkStats();
    },

    /**
     * Toggle custom period form
     */
    toggleCustomPeriod() {
        const form = document.getElementById('customPeriodForm');
        const isVisible = form.style.display !== 'none';

        if (isVisible) {
            form.style.display = 'none';
        } else {
            form.style.display = 'block';
            document.querySelectorAll('.quick-filter-btn').forEach(btn => btn.classList.remove('active'));
            document.querySelector('[data-period="custom"]').classList.add('active');
        }
    },

    /**
     * Apply custom period
     */
    async applyCustomPeriod() {
        const fromInput = document.getElementById('statsFromDate');
        const toInput = document.getElementById('statsToDate');

        const from = fromInput.value ? new Date(fromInput.value).toISOString() : '';
        const to = toInput.value ? new Date(toInput.value).toISOString() : '';

        if (!from || !to) {
            alert('Please select both start and end dates');
            return;
        }

        this.state.fromDate = from;
        this.state.toDate = to;
        this.state.currentPeriod = 'custom';
        this.state.currentPage = 1;

        await this.loadLinkStats();
    },

    /**
     * Load link statistics
     */
    async loadLinkStats() {
        try {
            // 1. Load table data (paginated)
            const tableParams = {
                page: this.state.currentPage,
                pageSize: this.state.pageSize
            };

            if (this.state.fromDate) tableParams.from = this.state.fromDate;
            if (this.state.toDate) tableParams.to = this.state.toDate;

            const tableData = await API.getLinkStats(this.code, tableParams);

            if (!tableData) return;

            // Update link info (only on first page)
            if (this.state.currentPage === 1) {
                const shortUrl = `https://${tableData.domain}/${tableData.code}`;
                document.getElementById('shortUrl').textContent = shortUrl;
                document.getElementById('shortUrl').href = shortUrl;
                document.getElementById('longUrl').textContent = tableData.longUrl;
                document.getElementById('longUrl').href = tableData.longUrl;
                document.getElementById('domain').textContent = tableData.domain;
                document.getElementById('totalClicks').textContent = tableData.total || 0;
                document.getElementById('createdAt').textContent = Utils.formatDateTime(tableData.createdAt);

                const copyBtn = document.getElementById('copyBtn');
                if (copyBtn) {
                    copyBtn.onclick = () => Utils.copyToClipboard(shortUrl, copyBtn);
                }
            }

            // 2. Load all clicks for chart (large page size)
            await this.loadAllClicksForChart();

            // 3. Render table and pagination
            this.renderClicksTable(tableData);
            this.state.totalPages = tableData.pagination?.totalPages || 1;
            this.renderPagination();
        } catch (error) {
            console.error('Failed to load stats:', error);
            Utils.showError(error.message, document.getElementById('clicksTable'));
        }
    },

    /**
     * Load all clicks for chart visualization
     */
    async loadAllClicksForChart() {
        try {
            const chartParams = {
                page: 1,
                pageSize: 1000 // Large page size for chart
            };

            if (this.state.fromDate) chartParams.from = this.state.fromDate;
            if (this.state.toDate) chartParams.to = this.state.toDate;

            const allData = await API.getLinkStats(this.code, chartParams);
            this.allClicksData = allData?.items || [];

            this.buildChartData();
            this.renderClicksChart();
        } catch (error) {
            console.error('Failed to load chart data:', error);
        }
    },

    /**
     * Build chart data from clicks
     */
    buildChartData() {
        if (!this.allClicksData || this.allClicksData.length === 0) {
            this.chartData = [];
            return;
        }

        const clicksByDate = {};
        this.allClicksData.forEach(click => {
            const clickDate = new Date(click.clickedAt);
            const localDateStr = `${clickDate.getFullYear()}-${String(clickDate.getMonth() + 1).padStart(2, '0')}-${String(clickDate.getDate()).padStart(2, '0')}`;
            clicksByDate[localDateStr] = (clicksByDate[localDateStr] || 0) + 1;
        });

        let startDate, endDate;

        if (this.state.fromDate && this.state.toDate) {
            startDate = new Date(this.state.fromDate);
            endDate = new Date(this.state.toDate);
        } else {
            // Default: last 30 days
            endDate = new Date();
            startDate = new Date();
            startDate.setDate(startDate.getDate() - 29);
        }

        this.chartData = [];
        const currentDate = new Date(startDate);

        while (currentDate <= endDate) {
            const dateStr = `${currentDate.getFullYear()}-${String(currentDate.getMonth() + 1).padStart(2, '0')}-${String(currentDate.getDate()).padStart(2, '0')}`;
            this.chartData.push({
                date: dateStr,
                clicks: clicksByDate[dateStr] || 0
            });
            currentDate.setDate(currentDate.getDate() + 1);
        }
    },

    /**
     * Render clicks chart using ECharts
     */
    renderClicksChart() {
        const chartDom = document.getElementById('clicksChart');

        if (!this.chartData || this.chartData.length === 0) {
            // Show empty state
            chartDom.innerHTML = '<p style="text-align: center; color: #999; padding: 20px;">No data available for the selected period</p>';
            chartDom.style.minHeight = 'auto';
            return;
        }

        chartDom.style.minHeight = '400px';
        chartDom.innerHTML = '';

        if (this.chart) {
            this.chart.dispose();
        }

        this.chart = echarts.init(chartDom);

        const option = {
            tooltip: {
                trigger: 'axis',
                backgroundColor: 'rgba(50, 50, 50, 0.9)',
                borderColor: '#ccc',
                textStyle: {
                    color: '#fff'
                }
            },
            grid: {
                left: '3%',
                right: '3%',
                top: '5%',
                bottom: '10%',
                containLabel: true
            },
            xAxis: {
                type: 'category',
                data: this.chartData.map(d => {
                    const date = new Date(d.date);
                    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
                }),
                boundaryGap: false
            },
            yAxis: {
                type: 'value',
                minInterval: 1
            },
            series: [{
                name: 'Clicks',
                type: 'line',
                data: this.chartData.map(d => d.clicks),
                smooth: true,
                itemStyle: {
                    color: '#2563eb'
                },
                areaStyle: {
                    color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        { offset: 0, color: 'rgba(37, 99, 235, 0.3)' },
                        { offset: 1, color: 'rgba(37, 99, 235, 0.1)' }
                    ])
                },
                lineStyle: {
                    color: '#2563eb',
                    width: 2
                }
            }]
        };

        this.chart.setOption(option);

        // Handle window resize
        const resizeHandler = () => {
            if (this.chart) {
                this.chart.resize();
            }
        };
        window.removeEventListener('resize', resizeHandler);
        window.addEventListener('resize', resizeHandler);
    },

    /**
     * Render clicks table
     */
    renderClicksTable(data) {
        const container = document.getElementById('clicksTable');

        if (!data || !data.items || data.items.length === 0) {
            Utils.showEmpty('No clicks recorded yet', container);
            return;
        }

        container.innerHTML = `
            <table>
                <thead>
                    <tr>
                        <th>Time</th>
                        <th>User Agent</th>
                        <th>Referer</th>
                        <th>IP</th>
                    </tr>
                </thead>
                <tbody>
                    ${data.items.map(item => `
                        <tr>
                            <td>${Utils.formatDateTime(item.clickedAt)}</td>
                            <td style="max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" ${item.userAgent ? `title="${item.userAgent}"` : ''}>
                                ${item.userAgent || '—'}
                            </td>
                            <td style="max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" ${item.referer ? `title="${item.referer}"` : ''}>
                                ${item.referer ? `<a href="${item.referer}" target="_blank">${item.referer}</a>` : '—'}
                            </td>
                            <td><code>${item.ip || '—'}</code></td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
            <div class="pagination-info">
                Showing ${data.items.length} of ${data.pagination?.totalItems || 0} clicks
            </div>
        `;
    },

    /**
     * Render pagination
     */
    renderPagination() {
        const container = document.getElementById('pagination');
        if (!container) return;

        const { currentPage, totalPages } = this.state;
        let html = '<div class="pagination">';

        html += `<button ${currentPage === 1 ? 'disabled' : ''} onclick="Stats.goToPage(${currentPage - 1})">← Previous</button>`;

        for (let i = 1; i <= totalPages; i++) {
            if (i === 1 || i === totalPages || (i >= currentPage - 2 && i <= currentPage + 2)) {
                html += `<button class="${i === currentPage ? 'active' : ''}" onclick="Stats.goToPage(${i})">${i}</button>`;
            } else if (i === currentPage - 3 || i === currentPage + 3) {
                html += `<button disabled>...</button>`;
            }
        }

        html += `<button ${currentPage === totalPages ? 'disabled' : ''} onclick="Stats.goToPage(${currentPage + 1})">Next →</button>`;
        html += '</div>';
        container.innerHTML = html;
    },

    /**
     * Go to specific page
     */
    async goToPage(page) {
        this.state.currentPage = page;
        await this.loadLinkStats();
    },

    /**
     * Initialize Stats page
     */
    async init(code) {
        this.code = code;
        await this.setQuickFilter('all');
    }
};

// ============================================
// LOGIN MODULE
// ============================================
let Login = {
    init() {
        const loginForm = document.getElementById('loginForm');
        if (!loginForm) return;

        loginForm.addEventListener('submit', this.handleLogin.bind(this));
    },

    async handleLogin(e) {
        e.preventDefault();

        const token = document.getElementById('token').value;
        const errorDiv = document.getElementById('error');
        const submitBtn = e.target.querySelector('button[type="submit"]');

        submitBtn.disabled = true;
        submitBtn.textContent = 'Logging in...';
        errorDiv.style.display = 'none';

        try {
            const response = await fetch('/api/health', {
                headers: {
                    'Authorization': `Bearer ${token}`
                }
            });

            if (response.ok) {
                // Save token in cookie
                document.cookie = `auth_token=${token}; path=/; max-age=2592000; SameSite=Strict`; // 30 days
                // Redirect to dashboard
                window.location.href = '/dashboard';
                return;
            }

            const errorMessage = response.status === 401
                ? 'Invalid token. Please check and try again.'
                : 'Login failed. Please try again later.';

            this.showError(errorDiv, submitBtn, errorMessage);
        } catch (error) {
            this.showError(errorDiv, submitBtn, `Connection error: ${error.message}`);
        }
    },

    showError(errorDiv, submitBtn, message) {
        errorDiv.textContent = message;
        errorDiv.style.display = 'block';
        submitBtn.disabled = false;
        submitBtn.textContent = 'Login';
    }
};

// ============================================
// EXPORT TO GLOBAL SCOPE
// ============================================
window.Auth = Auth;
window.Utils = Utils;
window.API = API;
window.Dashboard = Dashboard;
window.Links = Links;
window.Stats = Stats;
window.Login = Login;
