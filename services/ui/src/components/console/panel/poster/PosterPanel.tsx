import Poster from "./Poster";
import PosterHeader from "./PosterHeader";

const PosterPanel = (props) => {
  return (
    <>
      <PosterHeader
        config={props.config?.header}
        pathname={props.pathname}
        handleTitle={props.handleTitle}
        handleRedirect={props.handleRedirect}
      />
      <Poster
        config={props.config?.form}
        pathname={props.pathname}
        path_params={props.path_params}
        handleRedirect={props.handleRedirect}
      />
    </>
  );
};

export default PosterPanel;
