function openTab(evt, tabName) {
  var i, tabcontent, tablinks;

  tabcontent = document.getElementsByClassName("tab-content");
  for (i = 0; i < tabcontent.length; i++) {
    tabcontent[i].style.display = "none";
  }

  tablinks = document.getElementsByClassName("tab-button");
  for (i = 0; i < tablinks.length; i++) {
    tablinks[i].className = tablinks[i].className.replace(" active", "");
  }

  document.getElementById(tabName).style.display = "block";
  evt.currentTarget.className += " active";
}

// Logic for the slider value display
document.addEventListener('DOMContentLoaded', () => {
    const slider = document.getElementById('compression-slider');
    const sliderValue = document.getElementById('slider-value');
    const values = ['Store', 'Fast', 'Normal', 'Slow'];

    if (slider) {
        slider.oninput = function() {
            sliderValue.textContent = values[this.value];
        }
    }
});
