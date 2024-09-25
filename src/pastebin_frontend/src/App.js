import { html, render } from 'lit-html';
import { pastebin_backend } from 'declarations/pastebin_backend';
import logo from './logo2.svg';
import './styles.css'; // Import the CSS file

class App {
  pastes = [];

  constructor() {
    this.#fetchPastes(); // Fetch pastes on initialization
    this.#render();
  }

  // Method to handle paste submission
  #handleSubmit = async (e) => {
    e.preventDefault();
    const content = document.getElementById('content').value;
    if (content) {
      // Create a new paste
      await pastebin_backend.create_paste(content);
      // Fetch updated list of pastes after creation
      await this.#fetchPastes();
    }
    this.#render();
  };

  // Fetch all pastes from the backend
  #fetchPastes = async () => {
    this.pastes = await pastebin_backend.list_pastes();
    this.#render(); // Re-render after fetching
  };

  // Render method to display form and pastes
  #render() {
    let body = html`
      <main class="container">
        <header>
          <img src="${logo}" alt="DFINITY logo" class="logo" />
          <h1>Pastebin</h1>
        </header>
        <form action="#" class="paste-form">
          <label for="content">Enter your paste content: &nbsp;</label>
          <textarea id="content" alt="Paste Content" class="textarea"></textarea>
          <br />
          <button type="submit" class="submit-button">Create Paste</button>
        </form>
        <section id="pastes" class="paste-list">
          <h2>Pastes:</h2>
          <ul>
            ${this.pastes.map(
              (paste) => html`
                <li class="paste-item">
                  <strong>ID:</strong> ${paste.id} <br />
                  <strong>Content:</strong> ${paste.content}
                </li>
              `
            )}
          </ul>
        </section>
      </main>
    `;
    render(body, document.getElementById('root'));
    document
      .querySelector('form')
      .addEventListener('submit', this.#handleSubmit);
  }
}

export default App;
