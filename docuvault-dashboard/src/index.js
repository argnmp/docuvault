import React from 'react';
import ReactDOM from 'react-dom/client';
import createSagaMiddleware from '@redux-saga/core';

import Error from './Error';
import Login from './Login';
import List from './List';
import Write from './Write';
import Tag from './Tag';
import Sequence, {loader as sequenceLoader} from './Sequence';
import SequenceList, {loader as sequenceListLoader} from './SequenceList';
import Document, { loader as documentLoader } from './Document';
import Update, { loader as updateLoader } from './Update';

import { createBrowserRouter, RouterProvider } from 'react-router-dom';
import Layout from './Layout';

import './reset.css';

import {store} from "./state";
import { Provider } from 'react-redux';

const root = ReactDOM.createRoot(document.getElementById('root'));
const router = createBrowserRouter([
    {
        path: '/',
        element: <Layout>hello docuvault</Layout>,
        errorElement: <Error/>
    },
    {
        path: "/login",
        element: <Layout><Login/></Layout>,
        errorElement: <Error/>
    },
    {
        path: "/write",
        element: <Layout><Write/></Layout>,
        errorElement: <Error/>
    },
    
    {
        path: "/list",
        element: <Layout><List/></Layout>,
        errorElement: <Error/>
    },
    {
        path: "/document/:doc_id",
        element: <Layout><Document/></Layout>,
        loader: (p) => documentLoader({...p, state: store.getState()}),
        errorElement: <Error/>
    },
    {
        path: "/update/:doc_id",
        element: <Layout><Update/></Layout>,
        loader: (p) => updateLoader({...p, state: store.getState()}),
        errorElement: <Error/>
    },
    {
        path: "/sequence",
        element: <Layout><Sequence/></Layout>,
        loader: (p) => sequenceLoader({...p, state: store.getState()}),
        errorElement: <Error/>
    },
    {
        path: "/sequence/:seq_id",
        element: <Layout><SequenceList/></Layout>,
        loader: (p) => sequenceListLoader({...p, state: store.getState()}),
        errorElement: <Error/>
    }


]);

root.render(
  <React.StrictMode>
    <Provider store={store}>
      <RouterProvider router={router} />
    </Provider>
  </React.StrictMode>
);
