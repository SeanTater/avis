import { html, render, useState, useContext, createContext } from "./htm-preact.module.js";

const Queue = createContext([]);

const App = () => { 
    let [elements, setElements] = useState([
        {
            name: "Me at the zoo",
            author: "jawed",
            thumbnail: "/static/placeholder.png"
        },
        {
            name: "Never gonna give you up",
            author: "Rick Astley",
            thumbnail: "/static/placeholder.png"
        },
        {
            name: "One",
            author: "Person",
            thumbnail: "/static/placeholder.png"
        },
        {
            name: "One",
            author: "Person",
            thumbnail: "/static/placeholder.png"
        },
        {
            name: "One",
            author: "Person",
            thumbnail: "/static/placeholder.png"
        },
        {
            name: "One",
            author: "Person",
            thumbnail: "/static/placeholder.png"
        },
        {
            name: "One",
            author: "Person",
            thumbnail: "/static/placeholder.png"
        },
        {
            name: "One",
            author: "Person",
            thumbnail: "/static/placeholder.png"
        },
        {
            name: "One",
            author: "Person",
            thumbnail: "/static/placeholder.png"
        },
    ]);
    return html`
    <nav class="top"> <h1> Kibitz Home </h1> </nav>
    <${Queue.Provider} value=${[]}>
        <main>
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