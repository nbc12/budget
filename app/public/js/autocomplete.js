class Autocomplete {
    constructor(inputElement, items, onSelect) {
        console.log("Autocomplete initializing for:", inputElement.id);
        this.input = inputElement;
        this.items = items; // Array of { id, name } objects
        this.onSelect = onSelect;
        this.currentFocus = -1;
        this.listContainer = null;

        this.init();
    }

    init() {
        this.input.addEventListener("input", (e) => {
            this.renderList();
            this.validate();
        });
        this.input.addEventListener("keydown", (e) => this.onKeyDown(e));
        
        // Show all items on focus
        this.input.addEventListener("focus", () => {
            this.renderList();
            this.validate();
        });
        // Also show on click in case it's already focused
        this.input.addEventListener("click", () => {
            this.renderList();
            this.validate();
        });
        
        // Close list when clicking outside
        document.addEventListener("click", (e) => {
            if (e.target !== this.input) {
                this.closeList();
                this.validate();
            }
        });

        // Reposition on scroll/resize
        window.addEventListener("scroll", () => this.repositionList(), true);
        window.addEventListener("resize", () => this.repositionList());
    }

    validate() {
        const val = this.input.value;
        if (!val) {
            this.input.classList.remove("is-invalid");
            if (this.onSelect) this.onSelect(null);
            return;
        }

        const match = this.items.find(item => item.name.toLowerCase() === val.toLowerCase());
        if (match) {
            this.input.classList.remove("is-invalid");
            if (this.onSelect) this.onSelect(match);
        } else {
            this.input.classList.add("is-invalid");
            if (this.onSelect) this.onSelect(null);
        }
    }

    repositionList() {
        if (!this.listContainer) return;
        const rect = this.input.getBoundingClientRect();
        // Using FIXED positioning relative to viewport
        this.listContainer.style.position = "fixed";
        this.listContainer.style.top = rect.bottom + "px";
        this.listContainer.style.left = rect.left + "px";
        this.listContainer.style.width = rect.width + "px";
    }

    renderList() {
        const val = this.input.value;
        this.closeList();
        
        this.currentFocus = -1;
        this.listContainer = document.createElement("div");
        this.listContainer.setAttribute("class", "autocomplete-items");
        
        // Append to body to avoid clipping
        document.body.appendChild(this.listContainer);
        this.repositionList();

        const matches = val 
            ? this.items.filter(item => item.name.toLowerCase().includes(val.toLowerCase()))
            : this.items;

        if (matches.length === 0) {
            this.closeList();
            return;
        }

        for (const item of matches) {
            const div = document.createElement("div");
            if (val) {
                const regex = new RegExp(`(${val})`, "gi");
                div.innerHTML = item.name.replace(regex, "<strong>$1</strong>");
            } else {
                div.innerHTML = item.name;
            }
            div.innerHTML += `<input type='hidden' value='${item.id}'>`;
            
            div.addEventListener("click", () => {
                this.input.value = item.name;
                if (this.onSelect) this.onSelect(item);
                this.closeList();
                this.validate();
            });
            
            this.listContainer.appendChild(div);
        }
    }

    onKeyDown(e) {
        let x = this.listContainer;
        if (x) x = x.getElementsByTagName("div");
        
        if (e.keyCode == 40) { // Down
            this.currentFocus++;
            this.addActive(x);
        } else if (e.keyCode == 38) { // Up
            this.currentFocus--;
            this.addActive(x);
        } else if (e.keyCode == 13) { // Enter
            if (this.currentFocus > -1) {
                e.preventDefault();
                if (x) x[this.currentFocus].click();
            }
        } else if (e.keyCode == 9) { // Tab
            if (x && x.length > 0) {
                // If nothing is focused, pick the first one. 
                // If something is focused, pick that one.
                const index = this.currentFocus > -1 ? this.currentFocus : 0;
                x[index].click();
            }
        }
    }

    addActive(x) {
        if (!x) return;
        this.removeActive(x);
        if (this.currentFocus >= x.length) this.currentFocus = 0;
        if (this.currentFocus < 0) this.currentFocus = (x.length - 1);
        x[this.currentFocus].classList.add("autocomplete-active");
    }

    removeActive(x) {
        for (let i = 0; i < x.length; i++) {
            x[i].classList.remove("autocomplete-active");
        }
    }

    closeList() {
        if (this.listContainer) {
            this.listContainer.remove();
            this.listContainer = null;
        }
    }
}
