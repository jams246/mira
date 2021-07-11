export default class ListColumn {

    constructor(dataTableContainer)
    {
        this.dataTableContainer = dataTableContainer;
    }

    attachListener(listElement) {
        listElement.addEventListener("change", (event) => {
            this.updateDataTable(event.target.value)
        });
    }

    updateDataTable(selectedValue) {
        document.querySelectorAll(this.dataTableContainer +" td:not(:first-child):not(.mobileHidden)").forEach((element) => {
            element.classList.add('mobileHidden');
        });
        document.querySelectorAll(this.dataTableContainer +" td[data-id='"+ selectedValue +"']").forEach((element) => {
            element.classList.remove('mobileHidden');
        });
    }
}