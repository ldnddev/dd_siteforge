// dd cookie consent + Google Consent Mode v2.
// Accept grants analytics_storage ONLY; ad_storage, ad_user_data and ad_personalization are always 'denied'.
// Defaults (analytics 'denied', or 'granted' for a stored 'accept') are set inline in <head> (templates/_head.hbs)
// before gtag.js loads. This file only shows the banner and sends consent updates.
function dd_consent_update(state) {
  if (typeof window.gtag !== 'function') return;
  window.gtag('consent', 'update', {
    analytics_storage: state,
    ad_storage: 'denied',
    ad_user_data: 'denied',
    ad_personalization: 'denied'
  });
}
function dd_cookie_consent() {
  let consent = null;
  try { consent = localStorage.getItem('dd_cookie_consent'); } catch (e) {}
  const popup = document.getElementById('cookie-consent');
  if (!popup) return;
  // Show the banner only when no choice has been stored yet.
  popup.style.display = consent ? 'none' : 'block';
}
function handleCookieConsent(choice) {
  try { localStorage.setItem('dd_cookie_consent', choice); } catch (e) {}
  const popup = document.getElementById('cookie-consent');
  if (popup) popup.style.display = 'none';
  dd_consent_update(choice === 'accept' ? 'granted' : 'denied');
}

// Initialize on initial page load
document.addEventListener('DOMContentLoaded', () => {
  dd_cookie_consent();
});
