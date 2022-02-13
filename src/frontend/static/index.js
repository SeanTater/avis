import { html, render, useState, useContext, createContext } from "./htm-preact.module.js";

const Queue = createContext([]);

const App = () => { 
    let [elements, setElements] = useState([
        {
            name: "Regional sales word cloud",
            author: "Sean",
            thumbnail: "./static/screenshots/word-cloud-example.png"
        },
        {
            name: "Store simulation",
            author: "Sean",
            thumbnail: "./static/screenshots/word-cloud-example.png"
        },
        {
            name: "Live transactions map",
            author: "Sean",
            thumbnail: "./static/screenshots/word-cloud-example.png"
        }
    ]);
    return html`
    <nav class="top"> <h1> Visual Manager </h1> </nav>
    <${Queue.Provider} value=${[]}>
        <main>
            <h2>Select a visual</h2>
            <${MediaGrid} elements=${elements} />
        </main>
    </${Queue.Provider}>
    <${Navside} />
    `
}

const MediaGrid = ({elements}) => {
    return html`<card-deck>
        ${elements.map(e => VideoTile(e))}
    </card-deck>`
}

const VideoTile = ({id, name, thumbnail}) => html`
    <card>
        <img src=${thumbnail} />
        <p class="section">
            ${name}
        </p>
    </card>
`

const Navside = () => html`
    <nav class="side">
        <nav-item> Visualizations </nav-item>
        <nav-item> Settings </nav-item>
        <hr />
        <nav-item>
            Logged in as Anonymous
        </nav-item>
    </nav>
`


render(html`<${App} />`, document.body)