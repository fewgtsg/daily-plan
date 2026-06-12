import { BrowserRouter, Routes, Route, useLocation } from 'react-router-dom';
import { AnimatePresence } from 'framer-motion';
import { Layout } from './components/Layout';
import { ToastProvider } from './components/ToastProvider';
import { AnimatedPage } from './components/AnimatedPage';
import { RecordPage } from './pages/RecordPage';
import { BoardPage } from './pages/BoardPage';
import { SearchPage } from './pages/SearchPage';

function AnimatedRoutes() {
  const location = useLocation();
  return (
    <AnimatePresence mode="wait">
      <Routes location={location} key={location.pathname}>
        <Route element={<Layout />}>
          <Route path="/" element={<AnimatedPage><RecordPage /></AnimatedPage>} />
          <Route path="/board" element={<AnimatedPage><BoardPage /></AnimatedPage>} />
          <Route path="/search" element={<AnimatedPage><SearchPage /></AnimatedPage>} />
        </Route>
      </Routes>
    </AnimatePresence>
  );
}

function App() {
  return (
    <ToastProvider>
      <BrowserRouter>
        <AnimatedRoutes />
      </BrowserRouter>
    </ToastProvider>
  );
}

export default App;
