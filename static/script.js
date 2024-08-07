function checkState() {
  fetch('./state')
    .then(response => response.json())
    .then(data => {
      document.querySelector('#powerToggle').checked = data.power;
      document.querySelector('#colorPicker').value = '#' + data.color.map(x => x.toString(16).padStart(2, '0')).join('');
    });
}

function debounce_leading(func, timeout = 300){
  let timer;
  return (...args) => {
    if (!timer) {
      func(...args);
    }
    clearTimeout(timer);
    timer = setTimeout(() => {
      timer = undefined;
    }, timeout);

    return false;
  };
}

document.addEventListener('DOMContentLoaded', function() {
  document.querySelector('#powerToggle').addEventListener('click', debounce_leading(function() {
    fetch('./toggle_power', {method: 'POST'});
    return false;
  }));

  document.querySelector('#colorPicker').addEventListener('input', debounce_leading(function() {
    let color = document.querySelector('#colorPicker').value.match(/[A-Za-z0-9]{2}/g).map(x => parseInt(x, 16));
    fetch(`./set_color/${color[0]}/${color[1]}/${color[2]}`, {method: 'POST'});
  }));

  let params = new URLSearchParams(window.location.search);
  if (params.has('watch')) {
    setInterval(checkState, 1000);
  };
});