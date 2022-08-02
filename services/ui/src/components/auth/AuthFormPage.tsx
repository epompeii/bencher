import { Link, Navigate } from "solid-app-router";
import { createSignal, createEffect, Accessor } from "solid-js";

import authForms from "./authForms";
import { AuthForm } from "./AuthForm";

const SIGNUP = "signup";
const LOGIN = "login";

const AuthFormPage = (props: {
  kind: "signup" | "login";
  handleTitle: Function;
  handleRedirect: Function;
  user: Function;
  handleUser: Function;
  handleNotification: Function;
}) => {
  return (
    <section class="section">
      <div class="container">
        <div class="columns is-centered">
          <div class="column is-two-fifths">
            <h2 class="title">
              <span>{authForms[props.kind]?.heading}</span>
            </h2>

            <AuthForm
              kind={props.kind}
              handleTitle={props.handleTitle}
              handleRedirect={props.handleRedirect}
              user={props.user}
              handleUser={props.handleUser}
              handleNotification={props.handleNotification}
            />

            <hr />

            <p class="has-text-centered">
              <small>
                switch to{" "}
                {props.kind === SIGNUP && (
                  <Link href="/auth/login">log in</Link>
                )}
                {props.kind === LOGIN && (
                  <Link href="/auth/signup">sign up</Link>
                )}
              </small>
            </p>
          </div>
        </div>
      </div>
    </section>
  );
};

export default AuthFormPage;
