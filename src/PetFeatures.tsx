import "./characters.css";

/** Shared anatomy keeps the desktop pet and every preview in sync. */
export default function PetFeatures() {
  return <span className="pet-features" aria-hidden="true">
    <span className="pet-crown crown-left" /><span className="pet-crown crown-right" />
    <span className="leaf leaf-left" /><span className="leaf leaf-center" /><span className="leaf leaf-right" />
    <span className="face"><span className="cheek cheek-left" /><span className="cheek cheek-right" /><span className="eye eye-left"><span className="pupil" /></span><span className="eye eye-right"><span className="pupil" /></span><span className="mouth" /></span>
    <span className="arm arm-left" /><span className="arm arm-right" /><span className="foot foot-left" /><span className="foot foot-right" />
    <span className="pet-detail" />
  </span>;
}
