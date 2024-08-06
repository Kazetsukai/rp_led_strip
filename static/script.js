document.addEventListener('DOMContentLoaded', function() {
  document.querySelector('#powerToggle').addEventListener('click', function() {
    fetch('/toggle_power', {method: 'POST'});
  });
});