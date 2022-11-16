import "./styles/styles.scss";

import {
  createSignal,
  createEffect,
  lazy,
  Component,
  createMemo,
  Accessor,
  Signal,
  For,
} from "solid-js";
import { Routes, Route, Navigate, useLocation } from "solid-app-router";

import { Navbar } from "./components/site/navbar/Navbar";
import { site_analytics } from "./components/site/site_analytics";
import SiteFooter from "./components/site/pages/SiteFooter";
import { projectSlug } from "./components/console/ConsolePage";
import { BENCHER_TITLE } from "./components/site/pages/LandingPage";
import { BENCHER_GITHUB_URL, BENCHER_USER_KEY } from "./components/site/util";
import validator from "validator";

const AuthRoutes = lazy(() => import("./components/auth/AuthRoutes"));
const LandingPage = lazy(() => import("./components/site/pages/LandingPage"));
const ConsoleRoutes = lazy(() => import("./components/console/ConsoleRoutes"));
const DocsRoutes = lazy(() => import("./components/docs/DocsRoutes"));
const LegalRoutes = lazy(() => import("./components/legal/LegalRoutes"));
const Repo = lazy(() => import("./components/site/Repo"));

const initUser = () => {
  return {
    user: {
      uuid: null,
      name: null,
      slug: null,
      email: null,
      admin: null,
      locked: null,
    },
    token: null,
  };
};

const initNotification = () => {
  return {
    status: null,
    text: null,
  };
};

const App: Component = () => {
  const [title, setTitle] = createSignal<string>(BENCHER_TITLE);
  const [redirect, setRedirect] = createSignal<null | string>();
  const [user, setUser] = createSignal(initUser());
  const [notification, setNotification] = createSignal(initNotification());

  const location = useLocation();
  const pathname = createMemo(() => location.pathname);

  const [organization_slug, setOrganizationSlug] = createSignal<null | String>(
    null
  );
  // The project slug can't be a resource because it isn't 100% tied to the URL
  const [project_slug, setProjectSlug] = createSignal<String>(
    projectSlug(pathname)
  );

  createEffect(() => {
    if (document.title !== title()) {
      document.title = title();
    }
  });

  const handleUser = (user) => {
    window.localStorage.setItem(BENCHER_USER_KEY, JSON.stringify(user));
    setUser(user);
  };

  const removeUser = (user) => {
    window.localStorage.clear();
    setUser(initUser());
  };

  const removeNotification = () => {
    setNotification(initNotification());
  };

  const handleNotification = (notification: {
    status: string;
    text: string;
  }) => {
    setNotification(notification);
    setTimeout(() => {
      removeNotification();
    }, 4000);
  };

  setInterval(() => {
    if (user()?.token === null) {
      const user = JSON.parse(window.localStorage.getItem(BENCHER_USER_KEY));
      // TODO properly validate entire user
      if (user?.token && validator.isJWT(user.token)) {
        setUser(user);
      }
    }
  }, 1000);

  const handleTitle = (new_title) => {
    const bencher_title = `${new_title} - Bencher`;
    if (title() !== bencher_title) {
      setTitle(bencher_title);
    }
  };

  const getRedirect = () => {
    const new_pathname = redirect();
    if (new_pathname === undefined) {
      return;
    }
    if (new_pathname !== pathname()) {
      setRedirect();
      return <Navigate href={new_pathname} />;
    }
  };

  const getNotification = () => {
    let color: string;
    switch (notification().status) {
      case "ok":
        color = "is-success";
        break;
      case "alert":
        color = "is-primary";
        break;
      case "error":
        color = "is-danger";
        break;
      default:
        color = "";
    }
    return (
      <div class={`notification ${color}`}>
        {notification().text}
        <button
          class="delete"
          onClick={(e) => {
            e.preventDefault();
            removeNotification();
          }}
        />
      </div>
    );
  };

  const analytics = createMemo(site_analytics);

  return (
    <>
      <Navbar
        user={user}
        organization_slug={organization_slug}
        project_slug={project_slug}
        handleRedirect={setRedirect}
        handleProjectSlug={setProjectSlug}
      />
      {getRedirect()}

      {notification().text !== null && (
        <section class="section">
          <div class="container">{getNotification()}</div>
        </section>
      )}

      <Routes>
        <Route
          path="/"
          element={
            <LandingPage
              user={user}
              handleTitle={setTitle}
              handleRedirect={setRedirect}
            />
          }
        />

        {/* Auth Routes */}
        <Route path="/auth">
          <AuthRoutes
            handleTitle={handleTitle}
            handleRedirect={setRedirect}
            user={user}
            handleUser={handleUser}
            removeUser={removeUser}
            handleNotification={handleNotification}
          />
        </Route>

        {/* Console Routes */}
        <Route path="/console">
          <ConsoleRoutes
            user={user}
            pathname={pathname}
            organization_slug={organization_slug}
            project_slug={project_slug}
            handleTitle={handleTitle}
            handleRedirect={setRedirect}
            handleOrganizationSlug={setOrganizationSlug}
            handleProjectSlug={setProjectSlug}
          />
        </Route>

        {/* Docs Routes */}
        <Route path="/docs">
          <DocsRoutes />
        </Route>

        {/* Auth Routes */}
        <Route path="/legal">
          <LegalRoutes handleTitle={handleTitle} />
        </Route>

        {/* GitHub repo shortcut */}
        <Route path="/repo" element={<Repo />} />
      </Routes>

      <For each={[...Array(12).keys()]}>{(_k, _i) => <br />}</For>
      <SiteFooter />
    </>
  );
};

export default App;
