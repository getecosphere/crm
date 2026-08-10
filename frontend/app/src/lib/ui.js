export const STATUS_COLORS = {
  UNASSIGNED: 'bg-slate-100 text-slate-600',
  NEW: 'bg-blue-100 text-blue-700',
  CONTACTED: 'bg-amber-100 text-amber-700',
  IN_PROGRESS: 'bg-indigo-100 text-indigo-700',
  SALE: 'bg-green-100 text-green-700',
  NO_SALE: 'bg-red-100 text-red-700',
};

export const ROLE_COLORS = {
  ADMIN: 'bg-purple-100 text-purple-700',
  SALES_REP: 'bg-blue-100 text-blue-700',
  PARTNER: 'bg-emerald-100 text-emerald-700',
};

export function statusBadge(status) {
  const key = String(status || '').toUpperCase();
  const color = STATUS_COLORS[key] || STATUS_COLORS.UNASSIGNED;
  const label = key.charAt(0) + key.slice(1).toLowerCase();
  return `<span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium ${color}">
    <span class="w-1.5 h-1.5 rounded-full ${color.split(' ')[0].replace('bg-', 'bg-')}"></span>${label}
  </span>`;
}

export function roleBadge(role) {
  const key = String(role || '').toUpperCase();
  const color = ROLE_COLORS[key] || 'bg-slate-100 text-slate-600';
  const label = { ADMIN: 'Administrator', SALES_REP: 'Sales Rep', PARTNER: 'Partner' }[key] || key;
  return `<span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${color}">${label}</span>`;
}

export function activeBadge(status) {
  const active = String(status || '').toLowerCase() !== 'inactive';
  return active
    ? '<span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-700"><span class="w-1.5 h-1.5 rounded-full bg-green-500"></span>Active</span>'
    : '<span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-medium bg-slate-100 text-slate-600"><span class="w-1.5 h-1.5 rounded-full bg-slate-400"></span>Inactive</span>';
}

export function formatDate(value) {
  if (!value) return '—';
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return '—';
  return d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' });
}

export function formatDateTime(value) {
  if (!value) return '—';
  const d = new Date(value);
  if (Number.isNaN(d.getTime())) return '—';
  return d.toLocaleDateString('en-GB', { day: 'numeric', month: 'short', year: 'numeric' }) +
    ' ' + d.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' });
}

export function timeAgo(value) {
  if (!value) return '—';
  const then = new Date(value).getTime();
  if (Number.isNaN(then)) return '—';
  const seconds = Math.floor((Date.now() - then) / 1000);
  if (seconds < 60) return 'just now';
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}d ago`;
  return formatDate(value);
}

export function formatMoney(value) {
  if (value === null || value === undefined || value === '') return '—';
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
    maximumFractionDigits: 0,
  }).format(Number(value));
}

let toastTimer = null;
export function toast(message, type = 'success') {
  const palette = {
    success: 'bg-green-600',
    error: 'bg-red-600',
    info: 'bg-slate-800',
    warning: 'bg-amber-600',
  };
  let el = document.getElementById('toast-root');
  if (!el) {
    el = document.createElement('div');
    el.id = 'toast-root';
    el.className = 'fixed top-4 right-4 z-[100] flex flex-col gap-2';
    document.body.appendChild(el);
  }
  const toastEl = document.createElement('div');
  toastEl.className = `${palette[type] || palette.success} text-white text-sm font-medium px-4 py-3 rounded-lg shadow-lg max-w-sm animate-[fadeIn_0.2s_ease-out]`;
  toastEl.textContent = message;
  el.appendChild(toastEl);
  setTimeout(() => {
    toastEl.style.transition = 'opacity 0.3s';
    toastEl.style.opacity = '0';
    setTimeout(() => toastEl.remove(), 300);
  }, 4000);
}

export function loadingState(container, message = 'Loading…') {
  container.innerHTML = `
    <div class="flex items-center justify-center py-16">
      <div class="flex flex-col items-center gap-3">
        <div class="w-8 h-8 border-4 border-brand-200 border-t-brand-600 rounded-full animate-spin"></div>
        <p class="text-sm text-navy-500">${message}</p>
      </div>
    </div>`;
}

export function errorState(container, message) {
  container.innerHTML = `
    <div class="flex flex-col items-center justify-center py-16 text-center px-6">
      <div class="w-12 h-12 rounded-full bg-red-100 flex items-center justify-center mb-3">
        <svg class="w-6 h-6 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M12 9v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
      </div>
      <p class="text-sm font-medium text-navy-700">Something went wrong</p>
      <p class="text-sm text-navy-500 mt-1 max-w-md">${message}</p>
      <button class="mt-4 inline-flex items-center px-4 py-2 rounded-lg border border-navy-200 text-sm font-medium text-navy-700 hover:bg-navy-100" onclick="window.location.reload()">Try again</button>
    </div>`;
}

export function emptyState(container, title, hint, actionHtml = '') {
  container.innerHTML = `
    <div class="flex flex-col items-center justify-center py-16 text-center px-6">
      <div class="w-12 h-12 rounded-full bg-navy-100 flex items-center justify-center mb-3">
        <svg class="w-6 h-6 text-navy-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path stroke-linecap="round" stroke-linejoin="round" d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"/></svg>
      </div>
      <p class="text-sm font-medium text-navy-700">${title}</p>
      <p class="text-sm text-navy-500 mt-1 max-w-sm">${hint}</p>
      ${actionHtml ? `<div class="mt-4">${actionHtml}</div>` : ''}
    </div>`;
}

export function skeletonRows(container, rows = 5, cols = 5) {
  let html = '';
  for (let r = 0; r < rows; r++) {
    html += '<tr class="border-b border-navy-100">';
    for (let c = 0; c < cols; c++) {
      html += `<td class="px-4 py-3"><div class="h-4 rounded bg-navy-100 animate-pulse"></div></td>`;
    }
    html += '</tr>';
  }
  container.innerHTML = `<tbody>${html}</tbody>`;
}

export function escapeHtml(value) {
  if (value === null || value === undefined) return '';
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}
